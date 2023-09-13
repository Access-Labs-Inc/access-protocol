use solana_program::sysvar::Sysvar;
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    system_program,
    msg,
};

use crate::error::AccessError;
use crate::state::{CentralState, CentralStateV2};
use crate::utils::{check_account_key, check_account_owner};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `migrate_central_state_v2` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `migrate_central_state_v2` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The system program account
    pub system_program: &'a T,

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
            central_state: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
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
            accounts.central_state,
            program_id,
            AccessError::WrongOwner,
        )?;

        Ok(accounts)
    }
}

pub fn process_migrate_central_state_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralState::from_account_info(accounts.central_state)?;
    msg!("Central state v1: {:?}", central_state);

    // Migrate data
    let state_v2 = CentralStateV2::from_central_state(central_state);

    // Resize account
    let new_minimum_balance = Rent::get()?.minimum_balance(state_v2.borsh_len());
    let lamports_diff = new_minimum_balance
        .checked_sub(accounts.central_state.lamports())
        .ok_or(AccessError::Overflow)?;

    invoke(
        &system_instruction::transfer(accounts.fee_payer.key, accounts.central_state.key, lamports_diff),
        &[
            accounts.fee_payer.clone(),
            accounts.central_state.clone(),
            accounts.system_program.clone(),
        ],
    )?;
    accounts.central_state.realloc(state_v2.borsh_len(), false)?;

    // Save new data
    state_v2.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
