//! Sign a bond
//! This instruction is used by authorized sellers to approve the creation of a bond
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use crate::state:: CentralStateV2;
use crate::error::AccessError;
use crate::instruction::ProgramInstruction::SignBond;
use crate::state::{BOND_SIGNER_THRESHOLD, BondAccount, V1_INSTRUCTIONS_ALLOWED};
use crate::utils::{assert_authorized_seller, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `sign_bond` instruction
pub struct Params {
    seller_index: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `sign_bond` instruction
pub struct Accounts<'a, T> {
    #[cons(signer)]
    seller: &'a T,
    #[cons(writable)]
    bond_account: &'a T,
    central_state: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            seller: next_account_info(accounts_iter)?,
            bond_account: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.bond_account, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(accounts.seller, AccessError::BondSellerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_sign_bond(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    if !V1_INSTRUCTIONS_ALLOWED {
        return Err(AccessError::DeprecatedInstruction.into());
    }

    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(SignBond)?;
    let mut bond = BondAccount::from_account_info(accounts.bond_account, true)?;
    assert_authorized_seller(accounts.seller, params.seller_index as usize)?;

    if bond.sellers.len() == BOND_SIGNER_THRESHOLD as usize {
        msg!("There are enough signers already");
        return Err(AccessError::NoOp.into());
    }

    #[cfg(not(feature = "no-bond-signer"))]
    for current_seller in &bond.sellers {
        if accounts.seller.key == current_seller {
            msg!("The seller has already signed");
            return Err(AccessError::BondSellerAlreadySigner.into());
        }
    }

    bond.sellers.push(*accounts.seller.key);

    bond.save(&mut accounts.bond_account.data.borrow_mut())?;

    Ok(())
}
