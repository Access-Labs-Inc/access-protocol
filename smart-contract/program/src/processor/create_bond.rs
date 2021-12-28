//! Create a bond
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::cpi::Cpi;
use crate::error::AccessError;
use crate::state::{BondAccount, BOND_SIGNER_THRESHOLD};
use crate::utils::{
    assert_authorized_seller, assert_uninitialized, check_account_key, check_signer,
};
use bonfida_utils::{BorshSize, InstructionsAccount};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    pub buyer: Pubkey,
    pub total_amount_sold: u64,
    pub total_quote_amount: u64,
    pub quote_mint: Pubkey,
    pub seller_token_account: Pubkey,
    pub unlock_start_date: i64,
    pub unlock_period: i64,
    pub unlock_amount: u64,
    pub last_unlock_time: i64,
    pub stake_pool: Pubkey,
    pub seller_index: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The bond seller account
    #[cons(writable, signer)]
    pub seller: &'a T,

    /// The bond account
    #[cons(writable)]
    pub bond_account: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The fee account
    pub fee_payer: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            seller: next_account_info(accounts_iter)?,
            bond_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

        // Check signer
        check_signer(accounts.seller, AccessError::BondSellerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_create_bond(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let (derived_key, nonce) =
        BondAccount::create_key(&params.buyer, params.total_amount_sold, program_id);

    check_account_key(
        accounts.bond_account,
        &derived_key,
        AccessError::AccountNotDeterministic,
    )?;
    assert_uninitialized(accounts.bond_account)?;

    #[cfg(not(feature = "no-bond-signer"))]
    assert_authorized_seller(accounts.seller, params.seller_index as usize)?;

    let bond = BondAccount::new(
        params.buyer,
        params.total_amount_sold,
        params.total_quote_amount,
        params.quote_mint,
        params.seller_token_account,
        params.unlock_start_date,
        params.unlock_period,
        params.unlock_amount,
        params.last_unlock_time,
        params.stake_pool,
        params.unlock_start_date,
        *accounts.seller.key,
    );

    // Create bond account
    let seeds: &[&[u8]] = &[
        BondAccount::SEED.as_bytes(),
        &params.buyer.to_bytes(),
        &params.total_amount_sold.to_be_bytes(),
        &[nonce],
    ];

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.bond_account,
        seeds,
        bond.borsh_len() + ((BOND_SIGNER_THRESHOLD - 1) * 32) as usize,
    )?;

    bond.save(&mut accounts.bond_account.data.borrow_mut());

    Ok(())
}
