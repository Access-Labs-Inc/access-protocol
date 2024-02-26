//! Activate a stake pool
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::ActivateStakePool;
use crate::state::{CentralStateV2, StakePool, Tag};
use crate::utils::{check_account_owner};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The stake pool to activate
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The central state account
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
        check_account_owner(accounts.stake_pool, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;

        Ok(accounts)
    }
}

pub fn process_activate_stake_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::InactiveStakePool])?;
    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&ActivateStakePool)?;

    if stake_pool.header.tag != Tag::InactiveStakePool as u8 {
        return Err(AccessError::ActiveStakePoolNotAllowed.into());
    }

    stake_pool.header.tag = Tag::StakePool as u8;
    stake_pool.header.last_claimed_offset = central_state.last_snapshot_offset;
    if central_state.last_snapshot_offset > u16::MAX as u64 {
        return Err(AccessError::Overflow.into());
    }
    stake_pool.header.current_day_idx = central_state.last_snapshot_offset as u16;

    Ok(())
}
