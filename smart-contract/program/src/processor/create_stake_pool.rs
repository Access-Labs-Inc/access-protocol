use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    cpi::Cpi,
    error::MediaError,
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
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    #[cons(writable)]
    stake_pool_account: &'a T,
    system_program: &'a T,
    #[cons(writable, signer)]
    fee_payer: &'a T,
    rent_sysvar_account: &'a T,
    vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_pool_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            rent_sysvar_account: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            MediaError::WrongSystemProgram,
        )?;
        check_account_key(
            accounts.rent_sysvar_account,
            &sysvar::rent::ID,
            MediaError::WrongRent,
        )?;

        // Check ownership
        check_account_owner(
            accounts.stake_pool_account,
            &system_program::ID,
            MediaError::WrongOwner,
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
        MediaError::AccountNotDeterministic,
    )?;

    assert_valid_vault(accounts.vault, &derived_stake_key)?;

    let stake_pool_header = StakePoolHeader::new(
        params.owner,
        params.destination,
        params.nonce,
        *accounts.vault.key,
    );

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.stake_pool_account,
        accounts.rent_sysvar_account,
        &[
            params.name.as_bytes(),
            &params.owner.to_bytes(),
            &params.destination.to_bytes(),
            &[params.nonce],
        ],
        stake_pool_header.borsh_len() + 8 * STAKE_BUFFER_LEN as usize,
    )?;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool_account).unwrap();

    *stake_pool.header = stake_pool_header;

    Ok(())
}
