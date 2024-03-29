//! Admin program freeze instruction.
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::error::AccessError;
use crate::instruction::ProgramInstruction;
use crate::state::CentralStateV2;
use crate::utils::{check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `admin_program_freeze` instruction
pub struct Params {
    ///  The new ix gate
    pub ix_gate: u128,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `admin_program_freeze` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The central state account authority or freeze authority
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

pub fn process_admin_program_freeze(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params { ix_gate } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&ProgramInstruction::AdminProgramFreeze)?;

    // Only central state authority can unfreeze, the freeze authority is only allowed to freeze everything
    if accounts.authority.key != &central_state.authority && (accounts.authority.key != &central_state.freeze_authority || ix_gate > 0) {
        return Err(AccessError::WrongCentralStateAuthority.into());
    }

    central_state.ix_gate = ix_gate;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
