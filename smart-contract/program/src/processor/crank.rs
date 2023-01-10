//! Permissionless crank to update the stake pool rewards
//! This instructions updates the circular buffer with the pool balances multiplied by the current inflation

use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::error::AccessError;
use crate::state::{CentralState, RewardsTuple, SECONDS_IN_DAY, StakePool, Tag};
use crate::utils::check_account_owner;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `crank` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `crank` instruction
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The account of the central state
    #[cons(writable)]
    pub central_state: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_pool: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakeAccountOwner,
        )?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;

        Ok(accounts)
    }
}

pub fn process_crank(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;
    let mut central_state = CentralState::from_account_info(accounts.central_state)?;

    let current_offset = central_state.get_current_offset();
    // check if we need to do a system wide snapshot
    if central_state.last_snapshot_offset < current_offset {
        central_state.total_staked_snapshot = central_state.total_staked;
        central_state.last_snapshot_offset = current_offset;
        central_state.save(&mut accounts.central_state.data.borrow_mut())?;
        // reset the delta for the pool that we are currently cranking. We don't want the history to influence our result
        stake_pool.header.last_delta_update_offset = current_offset;
        stake_pool.header.total_staked_delta = 0;
    }

    if stake_pool.header.current_day_idx as u64 == central_state.last_snapshot_offset {
        #[cfg(not(any(feature = "days-to-sec-10s", feature = "days-to-sec-15m")))]
        return Err(AccessError::NoOp.into());
    }

    msg!("Total staked in pool {}", stake_pool.header.total_staked);
    msg!("Daily inflation {}", central_state.daily_inflation);
    msg!("Total staked {}", central_state.total_staked);
    msg!(
        "Total staked delta {}",
        stake_pool.header.total_staked_delta
    );
    msg!(
        "Total staked snapshot {}",
        central_state.total_staked_snapshot
    );

    let total_staked_snapshot = match stake_pool.header.total_staked_delta.signum() {
        1 => stake_pool
            .header
            .total_staked
            .checked_sub(stake_pool.header.total_staked_delta.unsigned_abs()) // <- Actualy safe to cast as u64 here
            .ok_or(AccessError::Overflow)?,
        -1 => stake_pool
            .header
            .total_staked
            .checked_add(stake_pool.header.total_staked_delta.unsigned_abs())
            .ok_or(AccessError::Overflow)?,
        _ => stake_pool.header.total_staked,
    } as u128;

    let mut stakers_reward: u128 = 0;
    let mut pool_reward: u128 = 0;
    if total_staked_snapshot > 0 {
        // stakers_reward = [(pool_total_staked << 32) * inflation * stakers_part] / (100 * total_staked * pool_total_staked)
        stakers_reward = (total_staked_snapshot << 32)
            .checked_mul(central_state.daily_inflation as u128)
            .ok_or(AccessError::Overflow)?
            .checked_mul(stake_pool.header.stakers_part as u128)
            .ok_or(AccessError::Overflow)?
            .checked_div(100u128)
            .ok_or(AccessError::Overflow)?
            .checked_div(central_state.total_staked_snapshot as u128)
            .ok_or(AccessError::Overflow)?
            .checked_div(total_staked_snapshot)
            .ok_or(AccessError::Overflow)?;

        msg!("Stakers reward {}", stakers_reward);

        // pool_rewards = [(pool_total_staked << 32) * inflation * (100 - stakers_part)] / (100 * total_staked)
        pool_reward = (total_staked_snapshot << 32)
            .checked_mul(central_state.daily_inflation as u128)
            .ok_or(AccessError::Overflow)?
            .checked_mul(
                100u64
                    .checked_sub(stake_pool.header.stakers_part)
                    .ok_or(AccessError::Overflow)? as u128,
            )
            .ok_or(AccessError::Overflow)?
            .checked_div(100u128)
            .ok_or(AccessError::Overflow)?
            .checked_div(central_state.total_staked_snapshot as u128)
            .ok_or(AccessError::Overflow)?;

        msg!("Pool reward {}", pool_reward);
    } else {
        msg!("Zero rewards");
    }

    stake_pool.push_balances_buff(
        current_offset,
        stake_pool.header.current_day_idx as u64,
        RewardsTuple {
            pool_reward,
            stakers_reward,
        },
    )?;

    Ok(())
}
