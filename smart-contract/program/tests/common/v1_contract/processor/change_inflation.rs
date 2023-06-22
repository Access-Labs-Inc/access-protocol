//! Change central state inflation
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::common::v1_contract::{error::AccessError, state::CentralState};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::common::v1_contract::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `change_inflation` instruction
pub struct Params {
    // The new daily inflation token amount
    pub daily_inflation: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `change_inflation` instruction
pub struct Accounts<'a, T> {
    /// The account of the central state
    #[cons(writable)]
    pub central_state: &'a T,

    /// The account of the central state authority
    #[cons(signer)]
    pub authority: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(
            accounts.authority,
            AccessError::CentralStateAuthorityMustSign,
        )?;

        Ok(accounts)
    }
}

pub fn process_change_inflation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut central_state = CentralState::from_account_info(accounts.central_state)?;

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    central_state.daily_inflation = params.daily_inflation;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
