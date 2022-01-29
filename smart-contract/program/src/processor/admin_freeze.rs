//! Freeze and unfreeze a program account
//! This admin instruction can be dangereous 💀
use crate::error::AccessError;
use crate::state::{CentralState, Tag};
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// The account tag to assign to the account
    pub tag: Tag,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The central state authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The account to freeze (or unfreeze)
    #[cons(writable)]
    pub account_to_freeze: &'a T,

    /// The account of the central state
    pub central_state: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            authority: next_account_info(accounts_iter)?,
            account_to_freeze: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(
            accounts.account_to_freeze,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(
            accounts.authority,
            AccessError::CentralStateAuthorityMustSign,
        )?;

        Ok(accounts)
    }
}

pub fn process_admin_freeze(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let Params { tag } = params;

    let central_state = CentralState::from_account_info(accounts.central_state)?;

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    let mut data = accounts.account_to_freeze.data.borrow_mut();

    if data[0] == Tag::CentralState as u8 || tag == Tag::CentralState {
        // Make sure to never edit the central state
        msg!("Central state cannot be modified");
        return Err(AccessError::DataTypeMismatch.into());
    }

    data[0] = tag as u8;

    Ok(())
}
