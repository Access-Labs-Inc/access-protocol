//! Stake
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::transfer;

use crate::{
    state::{CentralState, FEES, Tag},
    utils::{assert_valid_fee, check_account_key, check_account_owner, check_signer},
};
use crate::error::AccessError;
use crate::state::{SECONDS_IN_DAY, StakeAccount, StakePool};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `stake` instruction
pub struct Params {
    // Amount to stake
    pub amount: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `stake` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state_account: &'a T,

    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The owner of the stake account
    #[cons(signer)]
    pub owner: &'a T,

    /// The source account of the stake tokens
    #[cons(writable)]
    pub source_token: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The stake pool vault account
    #[cons(writable)]
    pub vault: &'a T,

    /// The stake fee account
    #[cons(writable)]
    pub fee_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state_account: next_account_info(accounts_iter)?,
            stake_account: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            source_token: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            fee_account: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.central_state_account,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(
            accounts.stake_account,
            program_id,
            AccessError::WrongStakeAccountOwner,
        )?;
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.source_token,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        check_account_owner(
            accounts.vault,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;

        // Check signer
        check_signer(accounts.owner, AccessError::StakeAccountOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let Params { amount } = params;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;
    let mut central_state = CentralState::from_account_info(accounts.central_state_account)?;

    check_account_key(
        accounts.owner,
        &stake_account.owner,
        AccessError::StakeAccountOwnerMismatch,
    )?;
    check_account_key(
        accounts.stake_pool,
        &stake_account.stake_pool,
        AccessError::StakePoolMismatch,
    )?;
    check_account_key(
        accounts.vault,
        &Pubkey::new(&stake_pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    assert_valid_fee(accounts.fee_account, &central_state.authority)?;

    let fees = (amount * FEES) / 100;

    if amount == 0 {
        return Err(AccessError::CannotStakeZero.into());
    }

    if stake_account.stake_amount > 0
        && stake_account.last_claimed_offset < stake_pool.header.current_day_idx as u64
    {
        return Err(AccessError::UnclaimedRewards.into());
    }

    if (stake_pool.header.current_day_idx as u64) < central_state.last_snapshot_offset {
        return Err(AccessError::PoolMustBeCranked.into());
    }

    if stake_account.stake_amount == 0 {
        stake_account.last_claimed_offset = central_state.get_current_offset();
    }

    // Transfer tokens
    let transfer_instruction = transfer(
        &spl_token::ID,
        accounts.source_token.key,
        accounts.vault.key,
        accounts.owner.key,
        &[],
        amount,
    )?;
    invoke(
        &transfer_instruction,
        &[
            accounts.spl_token_program.clone(),
            accounts.source_token.clone(),
            accounts.vault.clone(),
            accounts.owner.clone(),
        ],
    )?;

    // Transfer fees
    let transfer_fees = transfer(
        &spl_token::ID,
        accounts.source_token.key,
        accounts.fee_account.key,
        accounts.owner.key,
        &[],
        fees,
    )?;
    invoke(
        &transfer_fees,
        &[
            accounts.spl_token_program.clone(),
            accounts.source_token.clone(),
            accounts.fee_account.clone(),
            accounts.owner.clone(),
        ],
    )?;

    if stake_account
        .stake_amount
        .checked_add(amount)
        .ok_or(AccessError::Overflow)?
        < std::cmp::min(
            stake_account.pool_minimum_at_creation,
            stake_pool.header.minimum_stake_amount,
        )
    {
        msg!(
            "The minimum stake amount must be > {}",
            stake_account.pool_minimum_at_creation
        );
        return Err(ProgramError::InvalidArgument);
    }

    // Update stake account
    stake_account.deposit(amount)?;
    stake_pool.header.deposit(amount)?;

    //Update central state
    central_state.total_staked = central_state
        .total_staked
        .checked_add(amount)
        .ok_or(AccessError::Overflow)?;

    // Save states
    stake_account.save(&mut accounts.stake_account.data.borrow_mut())?;
    central_state.save(&mut accounts.central_state_account.data.borrow_mut())?;

    Ok(())
}
