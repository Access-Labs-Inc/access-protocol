//! Create a bond
//! This instruction can be used by authorized sellers to create a bond
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};
use solana_program::program_pack::Pack;
use solana_program::sysvar::Sysvar;
use spl_token::instruction::transfer;
use spl_token::state::Account;

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::AddToBondV2;
use crate::state::{BondAccountV2, CentralStateV2, StakePool};
use crate::state::Tag;
use crate::utils::{assert_valid_fee, check_account_key, check_account_owner, check_signer};

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

    /// The central state ATA
    #[cons(writable)]
    pub central_state_vault: &'a T,

    /// The mint address of the ACCESS token
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
            central_state_vault: next_account_info(accounts_iter)?,
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
        check_account_owner(accounts.central_state_vault, &spl_token::ID, AccessError::WrongOwner)?;
        check_account_owner(
            accounts.bond_account_v2,
            program_id,
            AccessError::WrongStakeAccountOwner,
        )?;
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

pub fn process_add_to_bond_v2(
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
    let mut bond = BondAccountV2::from_account_info(accounts.bond_account_v2)?;
    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&AddToBondV2)?;

    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;

    let source_token_acc = Account::unpack(&accounts.source_token.data.borrow())?;
    if source_token_acc.mint != central_state.token_mint {
        return Err(AccessError::WrongMint.into());
    }
    if &source_token_acc.owner != accounts.from.key {
        return Err(AccessError::WrongOwner.into());
    }

    check_account_key(accounts.to, &bond.owner, AccessError::WrongBondAccountOwner)?;
    check_account_key(accounts.pool, &bond.pool, AccessError::StakePoolMismatch)?;
    check_account_key(
        accounts.pool_vault,
        &Pubkey::new(&pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    assert_valid_fee(accounts.central_state_vault, accounts.central_state.key)?;

    if amount == 0 {
        return Err(AccessError::CannotStakeZero.into());
    }

    if bond.amount > 0 && bond.last_claimed_offset < pool.header.current_day_idx as u64 {
        return Err(AccessError::UnclaimedRewards.into());
    }

    if (pool.header.current_day_idx as u64) < central_state.get_current_offset()? {
        msg!(
            "Pool must be cranked before adding to a bond, {}, {}",
            pool.header.current_day_idx,
            central_state.get_current_offset()?
        );
        return Err(AccessError::PoolMustBeCranked.into());
    }

    if bond.amount == 0 {
        bond.last_claimed_offset = central_state.get_current_offset()?;
    }

    let current_time = Clock::get()?.unix_timestamp;
    if bond.unlock_date.is_some() && current_time > bond.unlock_date.unwrap() {
        msg!("Cannot add to a bond that has already started unlocking");
        return Err(ProgramError::InvalidArgument);
    }

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
        accounts.central_state_vault.key,
        accounts.from.key,
        &[],
        central_state.calculate_fee(amount)?,
    )?;
    invoke(
        &transfer_fees,
        &[
            accounts.spl_token_program.clone(),
            accounts.source_token.clone(),
            accounts.central_state_vault.clone(),
            accounts.from.clone(),
        ],
    )?;

    // Update all the appropriate states
    bond.amount = bond
        .amount
        .checked_add(amount)
        .ok_or(AccessError::Overflow)?;
    bond.save(&mut accounts.bond_account_v2.data.borrow_mut())?;
    pool.header.deposit(amount)?;
    central_state.total_staked = central_state
        .total_staked
        .checked_add(amount)
        .ok_or(AccessError::Overflow)?;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
