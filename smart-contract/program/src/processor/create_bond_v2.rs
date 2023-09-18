//! Create a bond
//! This instruction can be used by authorized sellers to create a bond
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};
use spl_token::instruction::transfer;
use spl_token::state::Account;
use crate::state:: CentralStateV2;

use crate::error::AccessError;
use crate::state::{BondAccountV2, StakePool, BOND_SIGNER_THRESHOLD};
use crate::utils::{
    assert_uninitialized, assert_valid_fee, check_account_key, check_account_owner, check_signer,
};
use crate::{cpi::Cpi, state::Tag};
use crate::instruction::ProgramInstruction::CreateBondV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_bond` instruction
pub struct Params {
    /// Total amount of ACCESS tokens being sold
    pub amount: u64,
    /// The start date of the unlock
    pub unlock_date: Option<i64>,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create_bond` instruction
pub struct Accounts<'a, T> {
    /// The fee account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The bond seller account
    #[cons(writable, signer)]
    pub from: &'a T,

    /// From ATA
    #[cons(writable)]
    pub source_token: &'a T,

    /// The bond recipient wallet
    pub to: &'a T,

    /// The bond account
    #[cons(writable)]
    pub bond_account_v2: &'a T,

    /// The pool account
    #[cons(writable)]
    pub pool: &'a T,

    /// Central state
    #[cons(writable)]
    pub central_state: &'a T,

    /// The vault of the pool
    #[cons(writable)]
    pub pool_vault: &'a T,

    /// The stake fee account
    #[cons(writable)]
    pub fee_account: &'a T,

    #[cons(writable)]
    pub mint: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The system program account
    pub system_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            fee_payer: next_account_info(accounts_iter)?,
            from: next_account_info(accounts_iter)?,
            source_token: next_account_info(accounts_iter)?,
            to: next_account_info(accounts_iter)?,
            bond_account_v2: next_account_info(accounts_iter)?,
            pool: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            pool_vault: next_account_info(accounts_iter)?,
            fee_account: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(
            accounts.pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.source_token,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        check_account_owner(
            accounts.pool_vault,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;

        // Check signers
        // todo - is this really needed? Possibly checked by #[cons(signer)]
        check_signer(accounts.fee_payer, AccessError::BondSellerMustSign)?;
        check_signer(accounts.from, AccessError::BondSellerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_create_bond_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params {
        amount,
        unlock_date,
    } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut pool = StakePool::get_checked(accounts.pool, vec![Tag::StakePool])?;
    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&CreateBondV2)?;

    let (derived_key, nonce) =
        BondAccountV2::create_key(accounts.to.key, accounts.pool.key, unlock_date, program_id);
    check_account_key(
        accounts.pool_vault,
        &Pubkey::new(&pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    // todo we might want to allow this - or at least check it there are any other accounts and add up the amounts
    if pool.header.minimum_stake_amount > amount {
        return Err(AccessError::InvalidAmount.into());
    }

    assert_valid_fee(accounts.fee_account, accounts.central_state.key)?;
    check_account_key(
        accounts.bond_account_v2,
        &derived_key,
        AccessError::AccountNotDeterministic,
    )?;
    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;
    assert_uninitialized(accounts.bond_account_v2)?;

    let source_token_acc = Account::unpack(&accounts.source_token.data.borrow())?;
    if source_token_acc.mint != central_state.token_mint {
        return Err(AccessError::WrongMint.into());
    }
    if &source_token_acc.owner != accounts.from.key {
        return Err(AccessError::WrongOwner.into());
    }

    let bond = BondAccountV2::new(
        *accounts.to.key,
        *accounts.pool.key,
        pool.header.minimum_stake_amount,
        amount,
        unlock_date,
        central_state.last_snapshot_offset,
    );

    // Create bond account
    let seeds: &[&[u8]] = &[
        BondAccountV2::SEED,
        &accounts.to.key.to_bytes(),
        &accounts.pool.key.to_bytes(),
        &unlock_date.unwrap_or(0).to_le_bytes(),
        &[nonce],
    ];

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.bond_account_v2,
        seeds,
        bond.borsh_len() + ((BOND_SIGNER_THRESHOLD - 1) * 32) as usize,
    )?;

    bond.save(&mut accounts.bond_account_v2.data.borrow_mut())?;

    // Transfer the tokens to pool vault (or burn for forever bonds)
    if unlock_date.is_some() {
        let transfer_instruction = transfer(
            &spl_token::ID,
            accounts.source_token.key,
            accounts.pool_vault.key,
            accounts.from.key,
            &[],
            amount,
        )?;
        invoke(
            &transfer_instruction,
            &[
                accounts.spl_token_program.clone(),
                accounts.source_token.clone(),
                accounts.pool_vault.clone(),
                accounts.from.clone(),
            ],
        )?;
    } else {
        let burn_instruction = spl_token::instruction::burn(
            &spl_token::ID,
            accounts.source_token.key,
            accounts.mint.key,
            accounts.from.key,
            &[accounts.from.key],
            amount,
        )?;
        invoke(
            &burn_instruction,
            &[
                accounts.source_token.clone(),
                accounts.mint.clone(),
                accounts.from.clone(),
                accounts.from.clone(),
            ],
        )?;
    }

    // Transfer fees
    let transfer_fees = transfer(
        &spl_token::ID,
        accounts.source_token.key,
        accounts.fee_account.key,
        accounts.from.key,
        &[],
        central_state.calculate_fee(amount)?,
    )?;
    invoke(
        &transfer_fees,
        &[
            accounts.spl_token_program.clone(),
            accounts.source_token.clone(),
            accounts.fee_account.clone(),
            accounts.from.clone(),
        ],
    )?;

    // Update all the appropriate states
    pool.header.deposit(amount)?;
    central_state.total_staked = central_state
        .total_staked
        .checked_add(amount)
        .ok_or(AccessError::Overflow)?;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
