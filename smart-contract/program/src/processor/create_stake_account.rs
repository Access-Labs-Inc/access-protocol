use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program, sysvar,
    sysvar::Sysvar,
};

use crate::state::StakeAccount;
use crate::{cpi::Cpi, error::MediaError};

use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::utils::{check_account_key, check_account_owner};
#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    // The PDA nonce
    pub nonce: u8,
    // Owner of the stake account
    pub owner: Pubkey,
    // Stake pool
    pub stake_pool: Pubkey,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    #[cons(writable)]
    pub stake_account: &'a T,
    pub system_program: &'a T,
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    pub rent_sysvar_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            rent_sysvar_account: next_account_info(accounts_iter)?,
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
            accounts.stake_account,
            &system_program::ID,
            MediaError::WrongOwner,
        )?;

        Ok(accounts)
    }
}

pub fn process_create_stake_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;
    let derived_stake_key =
        StakeAccount::create_key(&params.nonce, &params.owner, &params.stake_pool, program_id);

    check_account_key(
        accounts.stake_account,
        &derived_stake_key,
        MediaError::AccountNotDeterministic,
    )?;

    let current_time = Clock::get()?.unix_timestamp;
    let stake_account = StakeAccount::new(params.owner, params.stake_pool, current_time);

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.stake_account,
        accounts.rent_sysvar_account,
        &[
            StakeAccount::SEED.as_bytes(),
            &params.owner.to_bytes(),
            &params.stake_pool.to_bytes(),
            &[params.nonce],
        ],
        stake_account.borsh_len(),
    )?;

    stake_account.save(&mut accounts.stake_account.data.borrow_mut());

    Ok(())
}
