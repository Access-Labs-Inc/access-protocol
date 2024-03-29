//! Stake
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_program::program_pack::Pack;
use spl_token::instruction::transfer;
use spl_token::state::Account;
use crate::state:: CentralStateV2;

use crate::{
    state::Tag,
    utils::{assert_valid_fee, check_account_key, check_account_owner, check_signer},
};
use crate::error::AccessError;
use crate::instruction::ProgramInstruction::Stake;
use crate::state::{StakeAccount, StakePool};

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
    pub central_state: &'a T,

    /// The destination stake account -  can be owned by anyone
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The owner of the token account - staker
    #[cons(signer)]
    pub token_owner: &'a T,

    /// The source account of the stake tokens
    #[cons(writable)]
    pub source_token: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The stake pool vault account
    #[cons(writable)]
    pub vault: &'a T,

    /// The central state ATA
    #[cons(writable)]
    pub central_state_vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state: next_account_info(accounts_iter)?,
            stake_account: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
            token_owner: next_account_info(accounts_iter)?,
            source_token: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            central_state_vault: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.central_state,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(
            accounts.central_state_vault,
            &spl_token::ID,
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
        check_signer(accounts.token_owner, AccessError::OwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params { amount } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;
    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&Stake)?;

    let source_token_acc = Account::unpack(&accounts.source_token.data.borrow())?;
    if source_token_acc.mint != central_state.token_mint {
        msg!("Invalid ACCESS mint");
        #[cfg(not(feature = "no-mint-check"))]
        return Err(AccessError::WrongMint.into());
    }
    if &source_token_acc.owner != accounts.token_owner.key {
        return Err(AccessError::WrongOwner.into());
    }

    check_account_key(
        accounts.stake_pool,
        &stake_account.stake_pool,
        AccessError::StakePoolMismatch,
    )?;
    check_account_key(
        accounts.vault,
        &Pubkey::from(stake_pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    // if we were previously under the minimum stake limit it gets reset to the pool's one
    if stake_account.stake_amount < stake_account.pool_minimum_at_creation {
        stake_account.pool_minimum_at_creation = stake_pool.header.minimum_stake_amount;
    }

    assert_valid_fee(accounts.central_state_vault, accounts.central_state.key)?;

    if amount == 0 {
        return Err(AccessError::CannotStakeZero.into());
    }

    if stake_account.stake_amount > 0
        && stake_account.last_claimed_offset < stake_pool.header.current_day_idx as u64
    {
        return Err(AccessError::UnclaimedRewards.into());
    }

    if (stake_pool.header.current_day_idx as u64) < central_state.get_current_offset()? {
        msg!(
            "Pool must be cranked before staking, {}, {}",
            stake_pool.header.current_day_idx,
            central_state.get_current_offset()?
        );
        return Err(AccessError::PoolMustBeCranked.into());
    }

    if stake_account.stake_amount == 0 {
        stake_account.last_claimed_offset = central_state.get_current_offset()?;
    }

    // Transfer tokens
    let transfer_instruction = transfer(
        &spl_token::ID,
        accounts.source_token.key,
        accounts.vault.key,
        accounts.token_owner.key,
        &[],
        amount,
    )?;
    invoke(
        &transfer_instruction,
        &[
            accounts.spl_token_program.clone(),
            accounts.source_token.clone(),
            accounts.vault.clone(),
            accounts.token_owner.clone(),
        ],
    )?;

    // Transfer fees
    let transfer_fees = transfer(
        &spl_token::ID,
        accounts.source_token.key,
        accounts.central_state_vault.key,
        accounts.token_owner.key,
        &[],
        central_state.calculate_fee(amount)?,
    )?;
    invoke(
        &transfer_fees,
        &[
            accounts.spl_token_program.clone(),
            accounts.source_token.clone(),
            accounts.central_state_vault.clone(),
            accounts.token_owner.clone(),
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
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
