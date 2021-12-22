use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    cpi::Cpi,
    error::MediaError,
    state::{StakePoolHeader, STAKE_BUFFER_LEN},
};
use crate::{
    state::{BondAccount, BOND_SIGNER_THRESHOLD},
};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::utils::{assert_bond_derivation, check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    pub bond_account: &'a T,
    pub buyer: &'a T,
    pub quote_token_source: &'a T,
    pub quote_token_destination: &'a T,
    pub spl_token_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            bond_account: next_account_info(accounts_iter)?,
            buyer: next_account_info(accounts_iter)?,
            quote_token_source: next_account_info(accounts_iter)?,
            quote_token_destination: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
        };

        // Check keys

        // Check ownership

        // Check signer
        check_signer(accounts.buyer, MediaError::BuyerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_claim_bond(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;
    let mut bond = BondAccount::from_account_info(accounts.bond_account, true)?;

    assert_bond_derivation(
        accounts.bond_account,
        accounts.buyer.key,
        bond.total_amount_sold,
        program_id,
    )?;

    if bond.sellers.len() < BOND_SIGNER_THRESHOLD as usize {
        msg!("Not enough sellers have signed");
        return Err(MediaError::NotEnoughSellers.into());
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
    );

    // Activate the bond account
    bond.activate();

    bond.save(&mut accounts.bond_account.data.borrow_mut());

    Ok(())
}
