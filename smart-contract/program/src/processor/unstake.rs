//! Unstake
use crate::{
    state::{CentralState, Tag},
    utils::{check_account_key, check_account_owner, check_signer},
};
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;
use solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::error::AccessError;
use crate::state::{BondAccount, StakeAccount, StakePool, UnstakeRequest};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `unstake` instruction
pub struct Params {
    // Amount to unstake
    pub amount: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `unstake` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state_account: &'a T,

    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The owner of the stake account
    #[cons(signer)]
    pub owner: &'a T,

    /// Optional bond account to be able to stake under the minimum
    pub bond_account: Option<&'a T>,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state_account: next_account_info(accounts_iter)?,
            stake_account: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            bond_account: next_account_info(accounts_iter).ok(),
        };

        // Check ownership
        check_account_owner(
            accounts.central_state_account,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(
            accounts.stake_account,
            program_id,
            AccessError::WrongStakeAccountOwner,
        )?;
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        if let Some(bond_account) = accounts.bond_account {
            check_account_owner(
                bond_account,
                program_id,
                AccessError::WrongTokenAccountOwner,
            )?
        }

        // Check signer
        check_signer(accounts.owner, AccessError::StakeAccountOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params { amount} = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;
    let mut central_state = CentralState::from_account_info(accounts.central_state_account)?;
    let current_time = Clock::get()?.unix_timestamp;

    if stake_account.last_claimed_offset < stake_pool.header.current_day_idx as u64 {
        return Err(AccessError::UnclaimedRewards.into());
    }
    if (stake_pool.header.current_day_idx as u64) < central_state.get_current_offset() {
        return Err(AccessError::PoolMustBeCranked.into());
    }

    check_account_key(
        accounts.owner,
        &stake_account.owner,
        AccessError::StakeAccountOwnerMismatch,
    )?;
    check_account_key(
        accounts.stake_pool,
        &stake_account.stake_pool,
        AccessError::StakePoolMismatch,
    )?;

    let mut amount_in_bonds: u64 = 0;
    if let Some(bond_account) = accounts.bond_account {
        let bond_account = BondAccount::from_account_info(bond_account, false)?;
        check_account_key(
            accounts.owner,
            &bond_account.owner,
            AccessError::WrongOwner,
        )?;
        check_account_key(
            accounts.stake_pool,
            &bond_account.stake_pool,
            AccessError::StakePoolMismatch,
        )?;

        amount_in_bonds = bond_account.total_staked;
    }

    if stake_pool.header.minimum_stake_amount < stake_account.pool_minimum_at_creation {
        stake_account.pool_minimum_at_creation = stake_pool.header.minimum_stake_amount
    }

    // Can unstake either above the minimum or everything - includes the bond account
    let new_total_in_pool = stake_account.stake_amount
        .checked_add(amount_in_bonds)
        .ok_or(AccessError::Overflow)?
        .checked_sub(amount)
        .ok_or(AccessError::Overflow)?;
    if stake_account.stake_amount != amount && new_total_in_pool < stake_account.pool_minimum_at_creation {
        return Err(AccessError::InvalidUnstakeAmount.into());
    }

    // Update stake account
    stake_account.withdraw(amount)?;
    stake_pool.header.withdraw(amount)?;

    // Add unstake request
    stake_account.add_unstake_request(UnstakeRequest::new(amount, current_time))?;

    // Save states
    stake_account.save(&mut accounts.stake_account.data.borrow_mut())?;

    //Update central state
    central_state.total_staked = central_state
        .total_staked
        .checked_sub(amount)
        .ok_or(AccessError::Overflow)?;
    central_state.save(&mut accounts.central_state_account.data.borrow_mut())?;

    Ok(())
}
