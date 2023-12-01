//! Close a royalty account
//! This instruction can be used to close a royalty account. The laports will be sent to the original fee payer
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::{
    check_account_key, check_account_owner, check_signer,
};
use bonfida_utils::BorshSize;
use bonfida_utils::InstructionsAccount;

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::CloseStakeAccount;
use crate::state::{RoyaltyAccount, V1_INSTRUCTIONS_ALLOWED};
use crate::state:: CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `close_royalty_account` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `close_royalty_account` instruction
pub struct Accounts<'a, T> {
    /// The royalty account
    #[cons(writable)]
    pub royalty_account: &'a T,

    /// The royalty payer of the royalty account
    #[cons(signer)]
    pub royalty_payer: &'a T,

    /// The account where the funds should be returned
    #[cons(writable)]
    pub rent_destination: &'a T,

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
            royalty_payer: next_account_info(accounts_iter)?,
            rent_destination: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.royalty_account, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(accounts.royalty_payer, AccessError::StakePoolOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_close_royalty_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    if !V1_INSTRUCTIONS_ALLOWED {
        return Err(AccessError::DeprecatedInstruction.into());
    }

    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&CloseStakeAccount)?;
    let mut royalty_account = RoyaltyAccount::from_account_info(accounts.royalty_account)?;

    check_account_key(
        accounts.royalty_payer,
        &royalty_account.royalty_payer,
        AccessError::WrongOwner,
    )?;

    check_account_key(
        accounts.rent_destination,
        &royalty_account.rent_payer,
        AccessError::WrongQuoteDestination,
    )?;

    royalty_account.close();
    royalty_account.save(&mut accounts.royalty_account.data.borrow_mut())?;

    let mut account_lamports = accounts.royalty_account.lamports.borrow_mut();
    let mut destination_lamports = accounts.rent_destination.lamports.borrow_mut();

    **destination_lamports += **account_lamports;
    **account_lamports = 0;

    Ok(())
}
