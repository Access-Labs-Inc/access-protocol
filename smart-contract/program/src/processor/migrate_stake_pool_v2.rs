//! Migrate stake pool V2
use std::cell::RefMut;

use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::FromPrimitive;
use bonfida_utils::{fp_math::safe_downcast};
use solana_program::{
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    entrypoint::ProgramResult,
    program::invoke_signed,
    account_info::AccountInfo,
    account_info::next_account_info,
    system_program,
    program_pack::Pack
};
use spl_token::{instruction::mint_to, state::Account};
use crate::{
    error::AccessError,
    state::{RewardsTuple, StakePoolHeader, Tag},
};
use crate::{state::StakePool, utils::assert_valid_vault};
use crate::state::{CentralState, STAKE_BUFFER_LEN_V1, StakeAccount};
use crate::state::StakePoolRef;
use crate::utils::{calc_reward_fp32_v2, check_account_key, check_account_owner};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `migrate_stake_pool_v2` instruction
pub struct Accounts<'a, T> {
    /// The stake pool
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The stake pool owner account
    #[cons(signer)]
    pub owner: &'a T,

    /// The rewards destination
    #[cons(writable)]
    pub rewards_destination: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The stake pool vault account
    pub vault: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The mint address of the ACCESS token
    #[cons(writable)]
    pub mint: &'a T,

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
            stake_pool: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            rewards_destination: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
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
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.rewards_destination,
            &spl_token::ID,
            AccessError::WrongOwner,
        )?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.mint, &spl_token::ID, AccessError::WrongOwner)?;


        Ok(accounts)
    }
}

fn calc_remaining_pool_rewards(
    current_offset: u64,
    last_claimed_offset: u64,
    stake_pool: &StakePoolRef,
) -> Result<u128, ProgramError> {
    let mut nb_days_to_claim = current_offset.saturating_sub(last_claimed_offset);
    msg!("Nb of days behind {}", nb_days_to_claim);
    msg!("Last claimed offset {}", last_claimed_offset);
    msg!("Current offset {}", current_offset);
    nb_days_to_claim = std::cmp::min(nb_days_to_claim, STAKE_BUFFER_LEN_V1);
    msg!("Nb of days to claim {}", nb_days_to_claim);
    if nb_days_to_claim == 0 {
        return Ok(0);
    }

    if current_offset > stake_pool.header.current_day_idx as u64 {
        return Err(AccessError::PoolMustBeCranked.into());
    }

    msg!("Stake pool current day idx wrapped {}", (stake_pool.header.current_day_idx as u64) % STAKE_BUFFER_LEN_V1);
    // Saturating as we don't want to wrap around when there haven't been sufficient cranks
    let mut i = (stake_pool.header.current_day_idx as u64).saturating_sub(nb_days_to_claim)
        % STAKE_BUFFER_LEN_V1;

    // Compute reward for all past days
    let mut reward: u128 = 0;
    loop {
        reward = reward
            .checked_add(stake_pool.balances[i as usize].pool_reward)
            .ok_or(AccessError::Overflow)?;
        i = (i + 1) % STAKE_BUFFER_LEN_V1;
        if i == (stake_pool.header.current_day_idx as u64) % STAKE_BUFFER_LEN_V1 {
            break;
        }
    }

    msg!("Reward is {}", reward);
    Ok(reward)
}

pub fn process_migrate_stake_pool_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let central_state = CentralState::from_account_info(accounts.central_state)?;

    let destination_token_acc = Account::unpack(&accounts.rewards_destination.data.borrow())?;
    if destination_token_acc.mint != central_state.token_mint {
        return Err(AccessError::WrongMint.into());
    }

    let mut v2_balances: Vec<u128>;
    {
        let mut stake_pool_v1 =
            StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool, Tag::InactiveStakePool])?;

        // check if pool is cranked
        if (stake_pool_v1.header.current_day_idx as u64) < central_state.get_current_offset()? {
            return Err(AccessError::PoolMustBeCranked.into());
        }

        // Safety checks
        check_account_key(
            accounts.owner,
            &Pubkey::new(&stake_pool_v1.header.owner),
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_key(
            accounts.mint,
            &central_state.token_mint,
            AccessError::WrongMint,
        )?;

        // check if the pool owner is the same as destination
        msg!("Account owner: {}", destination_token_acc.owner);
        if destination_token_acc.owner.to_bytes() != stake_pool_v1.header.owner {
            return Err(AccessError::WrongDestinationAccount.into());
        }

        // check if stake pool is already v2
        let current_tag = Tag::from_u8(stake_pool_v1.header.tag as u8).ok_or(ProgramError::InvalidAccountData)?;
        if Tag::version(&current_tag) == 2 {
            return Err(AccessError::AlreadyUpgradedV2.into());
        }

        // Calculate and claim rewards for the pool owner
        let remaining_rewards = calc_remaining_pool_rewards(
            central_state.last_snapshot_offset,
            stake_pool_v1.header.last_claimed_offset,
            &stake_pool_v1,
        )?;

        let remaining_rewards = safe_downcast(((remaining_rewards >> 31) + 1) >> 1).ok_or(AccessError::Overflow)?;

        msg!("Claiming pool rewards {}", remaining_rewards);

        // Transfer rewards
        let transfer_ix = mint_to(
            &spl_token::ID,
            accounts.mint.key,
            accounts.rewards_destination.key,
            accounts.central_state.key,
            &[],
            remaining_rewards,
        )?;
        invoke_signed(
            &transfer_ix,
            &[
                accounts.spl_token_program.clone(),
                accounts.central_state.clone(),
                accounts.mint.clone(),
                accounts.rewards_destination.clone(),
            ],
            &[&[&program_id.to_bytes(), &[central_state.signer_nonce]]],
        )?;

        // upgrade tag to v2
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
    // todo fix this mapping - it won't work in case we are over the STAKE_BUFFER_LEN_V1. Will have a wrong gap in data.
    for i in 0..STAKE_BUFFER_LEN_V1 {
        if i % 2 == 0 {
            stake_pool_v2.balances[(i as usize) / 2] = v2_balances[(i as usize) / 2];
        } else {
            stake_pool_v2.balances[i as usize] = 0;
        }
    }

    Ok(())
}