//! Admin renounce functionality
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey};

use crate::error::AccessError;
use crate::instruction::ProgramInstruction;
use crate::state::CentralStateV2;
use crate::utils::{check_account_key, check_account_owner, check_signer, get_freeze_mask, is_admin_renouncable_instruction};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `admin_renounce` instruction
pub struct Params {
    // a bitmask of the instruction to renounce.
    // can only include one instruction at a time due to safety reasons
    pub ix: ProgramInstruction,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `admin_renounce` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The central state account authority
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

pub fn process_admin_renounce(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params { ix } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    if !is_admin_renouncable_instruction(&ix) {
        return Err(AccessError::InvalidRenounceParams.into());
    }

    let renounce_mask = get_freeze_mask(vec![ix]);
    let admin_ix_gate = central_state.admin_ix_gate & renounce_mask;

    if admin_ix_gate == central_state.admin_ix_gate {
        return Err(AccessError::AlreadyRenounced.into());
    }

    central_state.admin_ix_gate = admin_ix_gate;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
