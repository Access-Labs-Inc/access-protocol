//! Freeze and unfreeze a program account
//! This admin instruction can be dangereous 💀
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::AdminFreeze;
use crate::state::{CentralStateV2, Tag, V1_INSTRUCTIONS_ALLOWED};
use crate::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The central state authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The account to freeze (or unfreeze)
    #[cons(writable)]
    pub account_to_freeze: &'a T,

    /// The central state account
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

pub fn process_admin_freeze(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    if !V1_INSTRUCTIONS_ALLOWED {
        return Err(AccessError::DeprecatedInstruction.into());
    }

    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&AdminFreeze)?;

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    let mut data = accounts.account_to_freeze.data.borrow_mut();

    let current_tag = Tag::from_u8(data[0]).ok_or(ProgramError::InvalidAccountData)?;
    let new_tag = Tag::opposite(&current_tag)?;

    data[0] = new_tag as u8;

    Ok(())
}
