//! Create stake pool
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::{
    cpi::Cpi,
    error::AccessError,
    state::{StakePoolHeader, STAKE_BUFFER_LEN},
};
use crate::{state::StakePool, utils::assert_valid_vault};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::utils::{check_account_key, check_account_owner};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    // The PDA nonce
    pub nonce: u8,
    // Name of the stake pool
    pub name: String,
    // Owner of the stake pool
    pub owner: Pubkey,
    // Destination of the rewards
    pub destination: Pubkey,
    // Minimum amount to stake
    pub minimum_stake_amount: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool_account: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The stake pool vault account
    pub vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_pool_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

        // Check ownership
        check_account_owner(
            accounts.stake_pool_account,
            &system_program::ID,
            AccessError::WrongOwner,
        )?;

        Ok(accounts)
    }
}

pub fn process_create_stake_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;

    let derived_stake_key = StakePool::create_key(
        &params.nonce,
        &params.owner,
        &params.destination,
        program_id,
    );

    check_account_key(
        accounts.stake_pool_account,
        &derived_stake_key,
        AccessError::AccountNotDeterministic,
    )?;

    assert_valid_vault(accounts.vault, &derived_stake_key)?;

    let stake_pool_header = StakePoolHeader::new(
        params.owner,
        params.destination,
        params.nonce,
        *accounts.vault.key,
        params.minimum_stake_amount,
    );

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.stake_pool_account,
        &[
            params.name.as_bytes(),
            &params.owner.to_bytes(),
            &params.destination.to_bytes(),
            &[params.nonce],
        ],
        stake_pool_header.borsh_len() + 16 * STAKE_BUFFER_LEN as usize,
    )?;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool_account).unwrap();

    *stake_pool.header = stake_pool_header;

    Ok(())
}
