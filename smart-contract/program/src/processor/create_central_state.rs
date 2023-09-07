//! Create central state
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::state::CentralState;
use crate::{cpi::Cpi, error::AccessError};

use crate::utils::{check_account_key, check_account_owner};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_central_state` instruction
pub struct Params {
    // Daily inflation in token amount
    pub daily_inflation: u64,
    // Authority
    pub authority: Pubkey,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create_central_state` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The mint of the ACCESS token
    pub mint: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

        // Check ownership
        check_account_owner(
            accounts.central_state,
            &system_program::ID,
            AccessError::WrongOwner,
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

    check_account_key(
        accounts.central_state,
        &derived_state_key,
        AccessError::AccountNotDeterministic,
    )?;

    let state = CentralState::new(
        nonce,
        params.daily_inflation,
        *accounts.mint.key,
        params.authority,
        0,
    )?;

    msg!("+ Creating central state");
    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.central_state,
        &[&program_id.to_bytes(), &[nonce]],
        state.borsh_len(),
    )?;

    state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
