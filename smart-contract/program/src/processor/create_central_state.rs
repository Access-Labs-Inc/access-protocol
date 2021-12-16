use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program, sysvar,
};

use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::state::CentralState;
use crate::{cpi::Cpi, error::MediaError};

use crate::utils::{assert_valid_vault, check_account_key, check_account_owner};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    // Daily inflation in token amount
    pub daily_inflation: u64,
    // Authority
    pub authority: Pubkey,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    #[cons(writable)]
    pub state_account: &'a T,
    pub system_program: &'a T,
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    pub rent_sysvar_account: &'a T,
    pub central_vault: &'a T,
    pub mint: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            state_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            rent_sysvar_account: next_account_info(accounts_iter)?,
            central_vault: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
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
            accounts.state_account,
            &system_program::ID,
            MediaError::WrongOwner,
        )?;

        Ok(accounts)
    }
}

pub fn process_create_central_state(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;
    let (derived_state_key, nonce) = CentralState::find_key(program_id);

    assert_valid_vault(accounts.central_vault, &derived_state_key)?;

    check_account_key(
        accounts.state_account,
        &derived_state_key,
        MediaError::AccountNotDeterministic,
    )?;

    let state = CentralState::new(
        nonce,
        params.daily_inflation,
        *accounts.central_vault.key,
        *accounts.mint.key,
        params.authority,
    );

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.state_account,
        accounts.rent_sysvar_account,
        &[&program_id.to_bytes(), &[nonce]],
        state.borsh_len(),
    )?;

    state.save(&mut accounts.state_account.data.borrow_mut());

    Ok(())
}
