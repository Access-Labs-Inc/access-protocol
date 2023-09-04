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

// todo possibly delete - obsolete
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

// todo possibly delete - obsolete
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

pub fn process_create_bond(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    // This instruction is not supported in V2 anymore
    Err(AccessError::UnsupportedInstruction.into())
}
