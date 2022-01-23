//! Claim a bond after it has been issued and signed
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::AccessError;
use crate::state::{BondAccount, BOND_SIGNER_THRESHOLD};
use bonfida_utils::{BorshSize, InstructionsAccount};
use spl_token;

use crate::utils::{assert_bond_derivation, check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The bond account
    #[cons(writable)]
    pub bond_account: &'a T,

    /// The account of the bond buyer
    #[cons(signer)]
    pub buyer: &'a T,

    /// The token account used to purchase the bond
    #[cons(writable)]
    pub quote_token_source: &'a T,

    /// The token account where the sell proceed is sent
    #[cons(writable)]
    pub quote_token_destination: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            bond_account: next_account_info(accounts_iter)?,
            buyer: next_account_info(accounts_iter)?,
            quote_token_source: next_account_info(accounts_iter)?,
            quote_token_destination: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::id(),
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(accounts.bond_account, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(accounts.buyer, AccessError::BuyerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_claim_bond(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut bond = BondAccount::from_account_info(accounts.bond_account, true)?;

    assert_bond_derivation(
        accounts.bond_account,
        accounts.buyer.key,
        bond.total_amount_sold,
        program_id,
    )?;

    check_account_key(
        accounts.quote_token_destination,
        &bond.seller_token_account,
        AccessError::WrongQuoteDestination,
    )?;

    if bond.sellers.len() < BOND_SIGNER_THRESHOLD as usize {
        msg!("Not enough sellers have signed");
        return Err(AccessError::NotEnoughSellers.into());
    }

    // Transfer tokens
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        accounts.quote_token_source.key,
        accounts.quote_token_destination.key,
        accounts.buyer.key,
        &[],
        bond.total_quote_amount,
    )?;
    invoke(
        &transfer_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.quote_token_destination.clone(),
            accounts.quote_token_source.clone(),
            accounts.buyer.clone(),
        ],
    )?;

    // Activate the bond account
    bond.activate();

    bond.save(&mut accounts.bond_account.data.borrow_mut());

    Ok(())
}
