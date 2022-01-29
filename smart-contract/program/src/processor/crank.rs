//! Permissionless crank to update the stake pool rewards
//! This instructions updates the circular buffer with the pool balances multiplied by the current inflation
use crate::error::AccessError;
use crate::state::{CentralState, StakePool, SECONDS_IN_DAY};
use crate::utils::check_account_owner;
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

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

    let present_time = Clock::get()?.unix_timestamp;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, false)?;
    let central_state = CentralState::from_account_info(accounts.central_state)?;

    if present_time - stake_pool.header.last_crank_time < SECONDS_IN_DAY as i64 {
        #[cfg(not(feature = "no-lock-time"))]
        return Err(AccessError::NoOp.into());
    }
    msg!("Total staked in pool {}", stake_pool.header.total_staked);
    msg!("Daily inflation {}", central_state.daily_inflation);
    msg!("Total staked {}", central_state.total_staked);

    stake_pool.push_balances_buff(
        ((stake_pool.header.total_staked as u128) << 32)
            .checked_mul(central_state.daily_inflation as u128)
            .ok_or(AccessError::Overflow)?
            .checked_div(central_state.total_staked as u128)
            .ok_or(AccessError::Overflow)?,
    );
    stake_pool.header.last_crank_time = present_time;

    Ok(())
}
