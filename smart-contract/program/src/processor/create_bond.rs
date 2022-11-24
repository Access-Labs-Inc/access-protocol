//! Create a bond
//! This instruction can be used by authorized sellers to create a bond
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::error::AccessError;
use crate::state::{BondAccount, StakePool, BOND_SIGNER_THRESHOLD};
use crate::utils::{
    assert_uninitialized, check_account_key, check_account_owner,
    check_signer,
};
#[cfg(not(feature = "no-bond-signer"))]
use crate::utils::{assert_authorized_seller};
use crate::{cpi::Cpi, state::Tag};
use bonfida_utils::{BorshSize, InstructionsAccount};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_bond` instruction
pub struct Params {
    /// Ultimate buyer of the bond
    pub buyer: Pubkey,
    /// Total amount of ACCESS tokens being sold
    pub total_amount_sold: u64,
    /// Total price of the bond
    pub total_quote_amount: u64,
    /// Mint of the token used to buy the bond
    pub quote_mint: Pubkey,
    /// The token account i.e where the sell proceeds go
    pub seller_token_account: Pubkey,
    /// The start date of the unlock
    pub unlock_start_date: i64,
    /// The time interval at which the tokens unlock
    pub unlock_period: i64,
    /// The amount of tokens that unlock at each `unlock_period`
    pub unlock_amount: u64,
    /// Index of the seller in the [`array`][`crate::state::AUTHORIZED_BOND_SELLERS`] of authorized sellers
    pub seller_index: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create_bond` instruction
pub struct Accounts<'a, T> {
    /// The bond seller account
    #[cons(writable, signer)]
    pub seller: &'a T,

    /// The bond account
    #[cons(writable)]
    pub bond_account: &'a T,

    // The stake pool
    pub stake_pool: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The fee account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
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
            stake_pool: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

        // Check ownership
        check_account_owner(accounts.stake_pool, program_id, AccessError::WrongOwner)?;

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

    let stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;

    check_account_key(
        accounts.bond_account,
        &derived_key,
        AccessError::AccountNotDeterministic,
    )?;
    assert_uninitialized(accounts.bond_account)?;

    #[cfg(not(feature = "no-bond-signer"))]
    assert_authorized_seller(accounts.seller, params.seller_index as usize)?;

    if params.unlock_period == 0 {
        return Err(AccessError::ForbiddenUnlockPeriodZero.into());
    }

    let bond = BondAccount::new(
        params.buyer,
        params.total_amount_sold,
        params.total_quote_amount,
        params.quote_mint,
        params.seller_token_account,
        params.unlock_start_date,
        params.unlock_period,
        params.unlock_amount,
        params.unlock_start_date,
        stake_pool.header.minimum_stake_amount,
        *accounts.stake_pool.key,
        i64::MAX,
        *accounts.seller.key,
    );

    // Create bond account
    let seeds: &[&[u8]] = &[
        BondAccount::SEED,
        &params.buyer.to_bytes(),
        &params.total_amount_sold.to_le_bytes(),
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

    bond.save(&mut accounts.bond_account.data.borrow_mut())?;

    Ok(())
}
