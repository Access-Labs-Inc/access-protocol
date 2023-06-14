//! Migrate stake pool V2
use std::cell::RefMut;

use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::FromPrimitive;
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey, system_program};

use crate::{
    error::AccessError,
    state::{RewardsTuple, STAKE_BUFFER_LEN, StakePoolHeader, Tag},
};
use crate::{state::StakePool, utils::assert_valid_vault};
use crate::utils::{calc_reward_fp32, check_account_key, check_account_owner};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `migrate_stake_pool_v2` instruction
pub struct Accounts<'a, T> {
    /// The stake pool
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The stake pool vault account
    pub vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_pool: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongOwner,
        )?;

        Ok(accounts)
    }
}

pub fn process_migrate_stake_pool_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let central_state = CentralState::from_account_info(accounts.central_state)?;

    let mut v2_balances: Vec<u128>;
    {
        let mut stake_pool_v1 =
            StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool, Tag::InactiveStakePool])?;

        // todo claim pool rewards for the pool owner
        // (or maybe this should be called by pool owner)
        // (or maybe we should just throw them away)
        // (or maybe we should just require them to be claimed for the upgrade to go through)

        // check if stake pool is already v2
        let current_tag = Tag::from_u8(stake_pool_v1.header.tag as u8).ok_or(ProgramError::InvalidAccountData)?;
        if Tag::version(&current_tag) == 2 {
            return Err(AccessError::AlreadyUpgradedV2.into());
        }

        // upgrade to v2
        stake_pool_v1.header.tag = Tag::upgrade_v2(&current_tag)? as u8;

        // filter only stake rewards
        v2_balances = stake_pool_v1
            .balances
            .iter()
            .map(|RewardsTuple { stakers_reward, .. }| *stakers_reward)
            .collect();
        msg!("v2_balances: {:?}", v2_balances);
    }

    // write all items in v2 balances into stake pool v2 one by one and zero out the rest
    let mut stake_pool_v2 =
        StakePool::get_checked_v2(accounts.stake_pool, vec![Tag::StakePoolV2, Tag::InactiveStakePool])?;
    for i in 0..STAKE_BUFFER_LEN {
        if i % 2 == 0 {
            stake_pool_v2.balances[(i as usize) / 2] = v2_balances[(i as usize) / 2];
        } else {
            stake_pool_v2.balances[i as usize] = 0;
        }
    }

    Ok(())
}