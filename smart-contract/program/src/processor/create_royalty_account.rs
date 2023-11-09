//! Create royalty account
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::{cpi::Cpi, error::AccessError};
use crate::instruction::ProgramInstruction::CreateRoyaltyAccount;
use crate::state::CentralStateV2;
use crate::state::RoyaltyAccount;
use crate::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_royalty_account` instruction
pub struct Params {
    // Royalty basis points
    pub royalty_basis_points: u16,
    // Expiration date
    pub expiration_date: u64,
    // The ATA that should be getting the ACS rewards
    pub royalty_ata: Pubkey,
}

#[derive(InstructionsAccount)]
/// The required parameters for the `create_royalty_account` instruction
pub struct Accounts<'a, T> {
    /// The royalty account to be created
    #[cons(writable)]
    pub royalty_account: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The royalty payer
    #[cons(signer)]
    pub royalty_payer: &'a T,

    /// The system program account
    pub system_program: &'a T,

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
            royalty_account: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            royalty_payer: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(
            accounts.royalty_account,
            &system_program::ID,
            AccessError::WrongOwner,
        )?;

        check_signer(accounts.royalty_payer, AccessError::OwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_create_royalty_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    if params.royalty_basis_points > 10000 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&CreateRoyaltyAccount)?;

    let (derived_royalty_key, bump_seed) = RoyaltyAccount::create_key(
        &accounts.royalty_payer.key,
        program_id,
    );

    check_account_key(
        accounts.royalty_account,
        &derived_royalty_key,
        AccessError::AccountNotDeterministic,
    )?;

    let royalty_account = RoyaltyAccount::new(
        *accounts.fee_payer.key,
        *accounts.royalty_payer.key,
        params.royalty_ata,
        params.expiration_date,
        params.royalty_basis_points,
    );

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.royalty_account,
        &[
            RoyaltyAccount::SEED,
            &accounts.royalty_payer.key.to_bytes(),
            &[bump_seed],
        ],
        royalty_account.borsh_len(),
    )?;

    royalty_account.save(&mut accounts.royalty_account.data.borrow_mut())?;

    Ok(())
}
