//! Unstake
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use spl_token::instruction::transfer;

use crate::{
    state::{CentralState, StakePoolHeader},
    utils::{check_account_key, check_account_owner, check_signer},
};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::error::AccessError;
use crate::state::{StakeAccount, StakePool};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `unstake` instruction
pub struct Params {
    // Amount to unstake
    pub amount: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `unstake` instruction
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
        };

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

        // Check signer
        check_signer(accounts.owner, AccessError::StakeAccountOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let Params { amount } = params;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, false)?;
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

    // Update stake account
    stake_account.withdraw(amount)?;
    stake_pool.header.withdraw(amount)?;

    // Save states
    stake_account.save(&mut accounts.stake_account.data.borrow_mut());

    //Update central state
    central_state.total_staked = central_state
        .total_staked
        .checked_add(amount)
        .ok_or(AccessError::Overflow)?;
    central_state.save(&mut accounts.central_state_account.data.borrow_mut());

    Ok(())
}
