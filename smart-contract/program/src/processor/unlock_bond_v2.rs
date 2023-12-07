//! Unlock bond v2
use crate::{
    state::{Tag},
    utils::{check_account_key, check_account_owner, check_signer},
};
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::program::invoke_signed;
use solana_program::program_pack::Pack;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::transfer;
use spl_token::state::Account;

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::UnlockBondV2;
use crate::state::{BondV2Account, StakePool, StakePoolHeader};
use crate::state:: CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `unlock_bond_v2` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `unlock_bond_v2` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The bond account
    #[cons(writable)]
    pub bond_v2_account: &'a T,

    /// The owner of the bond V2 account
    #[cons(signer)]
    pub owner: &'a T,

    /// The destination of the locked tokens
    #[cons(writable)]
    pub destination_account: &'a T,

    /// The pool account
    #[cons(writable)]
    pub pool: &'a T,

    /// The pool vault
    #[cons(writable)]
    pub pool_vault: &'a T,

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
            central_state: next_account_info(accounts_iter)?,
            bond_v2_account: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            destination_account: next_account_info(accounts_iter)?,
            pool: next_account_info(accounts_iter)?,
            pool_vault: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.central_state,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(
            accounts.bond_v2_account,
            program_id,
            AccessError::WrongStakeAccountOwner,
        )?;
        check_account_owner(
            accounts.pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.destination_account,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        check_account_owner(
            accounts.pool_vault,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;

        // Check signer
        check_signer(accounts.owner, AccessError::BondSellerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_unlock_bond_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut stake_pool = StakePool::get_checked(accounts.pool, vec![Tag::StakePool])?;
    let mut bond_v2_account = BondV2Account::from_account_info(accounts.bond_v2_account)?;
    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&UnlockBondV2)?;

    let destination_token_acc = Account::unpack(&accounts.destination_account.data.borrow())?;
    if destination_token_acc.mint != central_state.token_mint {
        msg!("Invalid ACCESS mint");
        #[cfg(not(feature = "no-mint-check"))]
        return Err(AccessError::WrongMint.into());
    }

    if (stake_pool.header.current_day_idx as u64) < central_state.get_current_offset()? {
        return Err(AccessError::PoolMustBeCranked.into());
    }

    if bond_v2_account.last_claimed_offset < stake_pool.header.current_day_idx as u64 {
        return Err(AccessError::UnclaimedRewards.into());
    }

    check_account_key(
        accounts.owner,
        &bond_v2_account.owner,
        AccessError::StakeAccountOwnerMismatch,
    )?;
    check_account_key(
        accounts.pool,
        &bond_v2_account.pool,
        AccessError::StakePoolMismatch,
    )?;
    check_account_key(
        accounts.pool_vault,
        &Pubkey::from(stake_pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    if stake_pool.header.minimum_stake_amount < bond_v2_account.pool_minimum_at_creation {
        bond_v2_account.pool_minimum_at_creation = stake_pool.header.minimum_stake_amount
    }

    // Check if the bond can be unlocked
    if bond_v2_account.unlock_timestamp.is_none() {
        msg!("Cannot unlock a Forever bond");
        return Err(ProgramError::InvalidArgument);
    }
    let current_time = Clock::get()?.unix_timestamp;
    if current_time < bond_v2_account.unlock_timestamp.unwrap() {
        msg!("The bond tokens have not started unlocking yet");
        return Err(ProgramError::InvalidArgument);
    }

    let amount = bond_v2_account.amount;
    msg!("Unlocking {} tokens", amount);
    if amount == 0 {
        msg!("All tokens have been unlocked");
        return Err(ProgramError::InvalidArgument);
    }

    // Update bond v2 account
    bond_v2_account.withdraw(amount)?;
    stake_pool.header.withdraw(amount)?;

    // Transfer tokens
    let signer_seeds: &[&[u8]] = &[
        StakePoolHeader::SEED,
        &stake_pool.header.owner.clone(),
        &[stake_pool.header.nonce],
    ];
    let transfer_instruction = transfer(
        &spl_token::ID,
        accounts.pool_vault.key,
        accounts.destination_account.key,
        accounts.pool.key,
        &[],
        amount,
    )?;

    drop(stake_pool);

    invoke_signed(
        &transfer_instruction,
        &[
            accounts.spl_token_program.clone(),
            accounts.pool_vault.clone(),
            accounts.destination_account.clone(),
            accounts.pool.clone(),
        ],
        &[signer_seeds],
    )?;

    // Save states
    bond_v2_account.save(&mut accounts.bond_v2_account.data.borrow_mut())?;

    //Update central state
    central_state.total_staked = central_state
        .total_staked
        .checked_sub(amount)
        .ok_or(AccessError::Overflow)?;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
