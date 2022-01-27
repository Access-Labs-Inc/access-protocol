//! Unlock ACCESS tokens bought through a bond account
//! When tokens are unlocked they are withdrawn from the pool and are not considered staked anymore
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::error::AccessError;
use crate::state::{BondAccount, CentralState, StakePool, StakePoolHeader};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::utils::{assert_bond_derivation, check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `unlock_bond_tokens` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `unlock_bond_tokens` instruction
pub struct Accounts<'a, T> {
    /// The bond account
    #[cons(writable)]
    pub bond_account: &'a T,

    /// The account of the bond owner
    #[cons(signer)]
    pub bond_owner: &'a T,

    /// The ACCESS mint token
    pub mint: &'a T,

    /// The ACCESS token destination
    #[cons(writable)]
    pub access_token_destination: &'a T,

    /// The account of the central state
    #[cons(writable)]
    pub central_state: &'a T,

    /// The account of the staking pool
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The vault of the staking pool
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
            bond_account: next_account_info(accounts_iter)?,
            bond_owner: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            access_token_destination: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
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
        check_account_owner(accounts.bond_account, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.stake_pool, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(accounts.bond_owner, AccessError::BuyerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_unlock_bond_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut central_state = CentralState::from_account_info(accounts.central_state)?;
    let mut bond = BondAccount::from_account_info(accounts.bond_account, false)?;
    let mut stake_pool = StakePool::get_checked(accounts.stake_pool)?;
    let current_time = Clock::get()?.unix_timestamp;

    assert_bond_derivation(
        accounts.bond_account,
        accounts.bond_owner.key,
        bond.total_amount_sold,
        program_id,
    )?;
    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;
    check_account_key(
        accounts.pool_vault,
        &Pubkey::new(&stake_pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    if bond.total_amount_sold <= bond.total_unlocked_amount {
        msg!("All tokens have been unlocked");
        return Err(ProgramError::InvalidArgument);
    }

    if current_time < bond.unlock_start_date {
        msg!("The bond tokens have not started unlocking yet");
        return Err(ProgramError::InvalidArgument);
    }

    let delta = current_time
        .checked_sub(bond.last_unlock_time)
        .ok_or(AccessError::Overflow)?;

    if delta < bond.unlock_period {
        msg!("Need to wait the end of the current unlock period before unlocking the bond");
        return Err(ProgramError::InvalidArgument);
    }

    let missed_periods = delta
        .checked_div(bond.unlock_period)
        .ok_or(AccessError::Overflow)?;

    let unlock_amount = bond.calc_unlock_amount(missed_periods as u64)?;

    // Update the stake pool
    stake_pool.header.withdraw(unlock_amount)?;

    let signer_seeds: &[&[u8]] = &[
        StakePoolHeader::SEED.as_bytes(),
        &stake_pool.header.owner.clone(),
        &[stake_pool.header.nonce],
    ];

    drop(stake_pool);

    // Transfer tokens
    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        accounts.pool_vault.key,
        accounts.access_token_destination.key,
        accounts.stake_pool.key,
        &[],
        unlock_amount,
    )?;

    invoke_signed(
        &transfer_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.pool_vault.clone(),
            accounts.access_token_destination.clone(),
            accounts.stake_pool.clone(),
        ],
        &[signer_seeds],
    )?;

    // Update bond state
    bond.last_unlock_time += missed_periods * bond.unlock_period;
    bond.total_unlocked_amount += unlock_amount;
    bond.total_staked -= unlock_amount;

    bond.save(&mut accounts.bond_account.data.borrow_mut());

    // Update central state
    central_state.total_staked = central_state
        .total_staked
        .checked_add(unlock_amount)
        .ok_or(AccessError::Overflow)?;
    central_state.save(&mut accounts.central_state.data.borrow_mut());

    Ok(())
}
