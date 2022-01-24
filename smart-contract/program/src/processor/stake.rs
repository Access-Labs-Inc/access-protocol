//! Stake
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
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
    state::SECONDS_IN_DAY,
    utils::{check_account_key, check_account_owner, check_signer},
};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::error::AccessError;
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
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_account: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            source_token: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
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

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool)?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;

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

    let current_time = Clock::get().unwrap().unix_timestamp;
    if stake_account.stake_amount > 0
        && stake_account.last_claimed_time < current_time - (SECONDS_IN_DAY as i64)
    {
        return Err(AccessError::UnclaimedRewards.into());
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

    // Save states
    stake_account.save(&mut accounts.stake_account.data.borrow_mut());

    Ok(())
}
