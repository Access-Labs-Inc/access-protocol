//! Change the minimum stakeable amount of a pool
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::AccessError;
use crate::state::StakePool;
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    pub new_minimum: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The bond account
    #[cons(signer)]
    pub stake_pool_owner: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_pool: next_account_info(accounts_iter)?,
            stake_pool_owner: next_account_info(accounts_iter)?,
        };

        // Check keys

        // Check ownership
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;

        // Check signer
        check_signer(
            accounts.stake_pool_owner,
            AccessError::StakePoolOwnerMustSign,
        )?;

        Ok(accounts)
    }
}

pub fn process_change_pool_minimum(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let Params { new_minimum } = params;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool)?;

    check_account_key(
        accounts.stake_pool_owner,
        &Pubkey::new(&stake_pool.header.owner),
        AccessError::StakeAccountOwnerMismatch,
    )?;

    stake_pool.header.minimum_stake_amount = new_minimum;

    Ok(())
}

// This has repercusion on:
// - unstake.rs
// -  close_stake_account.rs
