//! Create a bond V2
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
    sysvar::Sysvar,
    clock::Clock,
    msg,
};
use spl_token::instruction::transfer;
use spl_token::state::Account;
use crate::state:: CentralStateV2;

use crate::error::AccessError;
use crate::state::{BondAccountV2, StakePool};
use crate::utils::{
    assert_uninitialized, assert_valid_fee, check_account_key, check_account_owner, check_signer,
};
use crate::{cpi::Cpi, state::Tag};
use crate::instruction::ProgramInstruction::CreateBondV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_bond_v2` instruction
pub struct Params {
    /// Total amount of ACCESS tokens being sold
    pub amount: u64,
    /// The timestamp of the unlock, if any
    pub unlock_timestamp: Option<i64>,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create_bond_v2` instruction
pub struct Accounts<'a, T> {
    /// The fee account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The bond seller account
    #[cons(writable, signer)]
    pub from: &'a T,

    /// From ATA
    #[cons(writable)]
    pub from_ata: &'a T,

    /// The bond recipient wallet
    pub to: &'a T,

    /// The bond account
    #[cons(writable)]
    pub bond_account_v2: &'a T,

    /// Central state
    #[cons(writable)]
    pub central_state: &'a T,

    /// The vault of the central state
    #[cons(writable)]
    pub central_state_vault: &'a T,

    /// The pool account
    #[cons(writable)]
    pub pool: &'a T,

    /// The vault of the pool
    #[cons(writable)]
    pub pool_vault: &'a T,

    /// The ACS token mint
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
            from_ata: next_account_info(accounts_iter)?,
            to: next_account_info(accounts_iter)?,
            bond_account_v2: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            central_state_vault: next_account_info(accounts_iter)?,
            pool: next_account_info(accounts_iter)?,
            pool_vault: next_account_info(accounts_iter)?,
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
            accounts.from_ata,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        check_account_owner(
            accounts.pool_vault,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;

        // Check signers
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
        unlock_timestamp,
    } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut pool = StakePool::get_checked(accounts.pool, vec![Tag::StakePool])?;
    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&CreateBondV2)?;
    assert_valid_fee(accounts.central_state_vault, accounts.central_state.key)?;

    check_account_key(
        accounts.pool_vault,
        &Pubkey::new(&pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    // We want to limit the bond amount by the pool minumum even in the case when the user has other subscriptions
    if pool.header.minimum_stake_amount > amount {
        return Err(AccessError::InvalidAmount.into());
    }

    let (derived_key, bump_seed) =
        BondAccountV2::create_key(accounts.to.key, accounts.pool.key, unlock_timestamp, program_id);

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

    let current_time = Clock::get()?.unix_timestamp;
    if unlock_timestamp.is_some() && current_time > unlock_timestamp.unwrap() {
        msg!("Cannot create a bond with an unlock timestamp in the past");
        return Err(ProgramError::InvalidArgument);
    }

    let from_ata = Account::unpack(&accounts.from_ata.data.borrow())?;
    if &from_ata.owner != accounts.from.key {
        return Err(AccessError::WrongOwner.into());
    }

    let bond = BondAccountV2::new(
        *accounts.to.key,
        *accounts.pool.key,
        pool.header.minimum_stake_amount,
        amount,
        unlock_timestamp,
        central_state.last_snapshot_offset,
    );

    // Create bond account
    let seeds: &[&[u8]] = &[
        BondAccountV2::SEED,
        &accounts.to.key.to_bytes(),
        &accounts.pool.key.to_bytes(),
        &unlock_timestamp.unwrap_or(0).to_le_bytes(),
        &[bump_seed],
    ];

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.bond_account_v2,
        seeds,
        bond.borsh_len(),
    )?;

    bond.save(&mut accounts.bond_account_v2.data.borrow_mut())?;

    // Transfer the tokens to pool vault (or burn for forever bonds)
    if unlock_timestamp.is_some() {
        let transfer_instruction = transfer(
            &spl_token::ID,
            accounts.from_ata.key,
            accounts.pool_vault.key,
            accounts.from.key,
            &[],
            amount,
        )?;
        invoke(
            &transfer_instruction,
            &[
                accounts.spl_token_program.clone(),
                accounts.from_ata.clone(),
                accounts.pool_vault.clone(),
                accounts.from.clone(),
            ],
        )?;
    } else {
        let burn_instruction = spl_token::instruction::burn(
            &spl_token::ID,
            accounts.from_ata.key,
            accounts.mint.key,
            accounts.from.key,
            &[accounts.from.key],
            amount,
        )?;
        invoke(
            &burn_instruction,
            &[
                accounts.from_ata.clone(),
                accounts.mint.clone(),
                accounts.from.clone(),
                accounts.from.clone(),
            ],
        )?;
    }

    // Transfer fees
    let transfer_fees = transfer(
        &spl_token::ID,
        accounts.from_ata.key,
        accounts.central_state_vault.key,
        accounts.from.key,
        &[],
        central_state.calculate_fee(amount)?,
    )?;
    invoke(
        &transfer_fees,
        &[
            accounts.spl_token_program.clone(),
            accounts.from_ata.clone(),
            accounts.central_state_vault.clone(),
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
