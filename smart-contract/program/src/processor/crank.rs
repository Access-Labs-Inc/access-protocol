use crate::error::MediaError;
use crate::state::{CentralState, StakePool, SECONDS_IN_DAY};
use crate::utils::check_account_owner;
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
struct Accounts<'a, T> {
    #[cons(writable)]
    stake_pool: &'a T,
    central_state: &'a T,
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
            MediaError::WrongStakeAccountOwner,
        )?;
        check_account_owner(accounts.central_state, program_id, MediaError::WrongOwner)?;

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

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool)?;
    let central_vault = CentralState::from_account_info(accounts.central_state)?;

    if present_time - stake_pool.header.last_crank_time < SECONDS_IN_DAY as i64 {
        #[cfg(not(feature = "no-lock-time"))]
        return Err(MediaError::NoOp.into());
    }

    // Maybe need u128?
    stake_pool.push_balances_buff(
        stake_pool
            .header
            .total_staked
            .checked_mul(central_vault.daily_inflation)
            .ok_or(MediaError::Overflow)?,
    );
    stake_pool.header.last_crank_time = present_time;

    Ok(())
}
