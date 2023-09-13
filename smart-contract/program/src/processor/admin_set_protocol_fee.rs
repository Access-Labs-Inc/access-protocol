//! Change central state inflation
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey, system_program};

use crate::{error::AccessError, state::FeeSplit};
use crate::instruction::ProgramInstruction::AdminSetProtocolFee;
use crate::state::CentralStateV2;
use crate::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `change_inflation` instruction
pub struct Params {
    // The new protocol fee basis points
    pub protocol_fee_basis_points: u16,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `change_inflation` instruction
pub struct Accounts<'a, T> {
    /// The central state authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The account of the central state
    pub central_state: &'a T,

    /// The fee split account
    #[cons(writable)]
    pub fee_split_pda: &'a T,

    /// The system program account
    pub system_program: &'a T,
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
            fee_split_pda: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.fee_split_pda, program_id, AccessError::WrongOwner)?;

        // Check account key
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

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

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(AdminSetProtocolFee)?;
    let mut fee_split = FeeSplit::from_account_info(accounts.fee_split_pda)?;

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    if protocol_fee_basis_points > 10000 {
        return Err(AccessError::InvalidAmount.into());
    }

    fee_split.fee_basis_points = protocol_fee_basis_points;
    fee_split.save(&mut accounts.fee_split_pda.data.borrow_mut())?;

    Ok(())
}
