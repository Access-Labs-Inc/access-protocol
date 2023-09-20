//! Admin set protocol fee
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey, system_program};

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::AdminSetProtocolFee;
use crate::state::CentralStateV2;
use crate::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `admin_set_protocol_fee` instruction
pub struct Params {
    // The new protocol fee basis points
    pub protocol_fee_basis_points: u16,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `admin_set_protocol_fee` instruction
pub struct Accounts<'a, T> {
    /// The central state authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The central state account
    #[cons(writable)]
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
            central_state: next_account_info(accounts_iter)?,
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

pub fn process_admin_set_protocol_fee(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params { protocol_fee_basis_points } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&AdminSetProtocolFee)?;

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    // Even though the protocol fee is added to the transaction (so we could do more than 100%),
    // we don't want to allow this.
    if protocol_fee_basis_points > 10000 {
        return Err(AccessError::InvalidAmount.into());
    }

    central_state.fee_basis_points = protocol_fee_basis_points;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
