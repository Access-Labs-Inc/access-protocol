use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::{assert_authorized_seller, check_account_owner, check_signer};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::error::MediaError;
use crate::state::{BondAccount, BOND_SIGNER_THRESHOLD};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    seller_index: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    #[cons(writable, signer)]
    seller: &'a T,
    #[cons(writable)]
    bond_account: &'a T,
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
        };

        // Check ownership
        check_account_owner(accounts.bond_account, program_id, MediaError::WrongOwner)?;

        // Check signer
        check_signer(accounts.seller, MediaError::BondSellerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_create_bond(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut bond = BondAccount::from_account_info(accounts.bond_account, true)?;
    assert_authorized_seller(accounts.seller, params.seller_index as usize)?;

    if bond.sellers.len() == BOND_SIGNER_THRESHOLD as usize {
        msg!("There are enough signers already");
        return Err(MediaError::NoOp.into());
    }

    for current_seller in &bond.sellers {
        if accounts.seller.key == current_seller {
            msg!("The seller has already signed");
            return Err(MediaError::BondSellerAlreadySigner.into());
        }
    }

    bond.sellers.push(*accounts.seller.key);

    bond.save(&mut accounts.bond_account.data.borrow_mut());

    Ok(())
}
