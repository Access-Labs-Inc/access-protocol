//! Claim bond
//! This instruction allows a buyer to claim a bond once it has been signed by enough DAO members.
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::state::{BondAccount, CentralState, StakePool, BOND_SIGNER_THRESHOLD};
use crate::{error::AccessError, state::Tag};
use bonfida_utils::{BorshSize, InstructionsAccount};
use spl_token;

use crate::utils::{assert_bond_derivation, check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `claim_bond` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `claim_bond_rewards` instruction
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

    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The mint of the ACCESS token
    #[cons(writable)]
    pub access_mint: &'a T,

    /// The vault of the stake pool
    #[cons(writable)]
    pub pool_vault: &'a T,

    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

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
            stake_pool: next_account_info(accounts_iter)?,
            access_mint: next_account_info(accounts_iter)?,
            pool_vault: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
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
        check_account_owner(accounts.stake_pool, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;

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
    let mut central_state = CentralState::from_account_info(accounts.central_state)?;
    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, Tag::StakePool)?;

    assert_bond_derivation(
        accounts.bond_account,
        accounts.buyer.key,
        bond.total_amount_sold,
        program_id,
    )?;

    check_account_key(
        accounts.stake_pool,
        &bond.stake_pool,
        AccessError::WrongStakePool,
    )?;
    check_account_key(
        accounts.access_mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;
    check_account_key(
        accounts.quote_token_destination,
        &bond.seller_token_account,
        AccessError::WrongQuoteDestination,
    )?;
    check_account_key(
        accounts.pool_vault,
        &Pubkey::new(&stake_pool.header.vault),
        AccessError::StakePoolVaultMismatch,
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
    let current_time = Clock::get()?.unix_timestamp;
    bond.activate(current_time);

    bond.save(&mut accounts.bond_account.data.borrow_mut());

    // Mint ACCESS tokens into the pool vault
    let mint_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        accounts.access_mint.key,
        accounts.pool_vault.key,
        accounts.central_state.key,
        &[],
        bond.total_amount_sold,
    )?;

    invoke_signed(
        &mint_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.access_mint.clone(),
            accounts.pool_vault.clone(),
            accounts.central_state.clone(),
        ],
        &[&[&program_id.to_bytes(), &[central_state.signer_nonce]]],
    )?;

    stake_pool.header.deposit(bond.total_amount_sold)?;

    // Update central state
    central_state.total_staked = central_state
        .total_staked
        .checked_add(bond.total_amount_sold)
        .ok_or(AccessError::Overflow)?;
    central_state.save(&mut accounts.central_state.data.borrow_mut());

    Ok(())
}
