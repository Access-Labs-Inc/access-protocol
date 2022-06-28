//! Create stake account
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
    sysvar::Sysvar,
};

use crate::state::{StakeAccount, StakePool, Tag};
use crate::{cpi::Cpi, error::AccessError};

use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::utils::{check_account_key, check_account_owner};
#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_stake_account` instruction
pub struct Params {
    // The PDA nonce
    pub nonce: u8,
    // Owner of the stake account
    pub owner: Pubkey,
}

#[derive(InstructionsAccount)]
/// The required parameters for the `create_stake_account` instruction
pub struct Accounts<'a, T> {
    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The stake pool account
    pub stake_pool: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

        // Check ownership
        check_account_owner(
            accounts.stake_account,
            &system_program::ID,
            AccessError::WrongOwner,
        )?;
        check_account_owner(accounts.stake_pool, program_id, AccessError::WrongOwner)?;

        Ok(accounts)
    }
}

pub fn process_create_stake_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;

    let derived_stake_key = StakeAccount::create_key(
        &params.nonce,
        &params.owner,
        accounts.stake_pool.key,
        program_id,
    );

    check_account_key(
        accounts.stake_account,
        &derived_stake_key,
        AccessError::AccountNotDeterministic,
    )?;

    let current_time = Clock::get()?.unix_timestamp;
    let stake_account = StakeAccount::new(
        params.owner,
        *accounts.stake_pool.key,
        current_time,
        stake_pool.header.minimum_stake_amount,
    );

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.stake_account,
        &[
            StakeAccount::SEED,
            &params.owner.to_bytes(),
            &accounts.stake_pool.key.to_bytes(),
            &[params.nonce],
        ],
        stake_account.borsh_len(),
    )?;

    stake_account.save(&mut accounts.stake_account.data.borrow_mut());

    Ok(())
}
