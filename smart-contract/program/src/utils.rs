use std::borrow::BorrowMut;
use std::convert::TryInto;

use crate::error::AccessError;
use crate::state::{BondAccount, RewardsTuple, StakePoolHeader, Tag, AUTHORIZED_BOND_SELLERS};
use crate::state::{StakeAccount, StakePoolRef, ACCESS_MINT, SECONDS_IN_DAY, STAKE_BUFFER_LEN};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

/// Cumulate the claimable rewards from the last claimed day to the present.
/// Result is in FP32 format.
///
/// * `staker` Compute the reward for a staker or a pool owner
pub fn calc_reward_fp32(
    current_time: i64,
    last_claimed_time: i64,
    stake_pool: &StakePoolRef,
    staker: bool,
) -> Result<u128, ProgramError> {
    let mut nb_days_to_claim =
        current_time.saturating_sub(last_claimed_time) as u64 / SECONDS_IN_DAY;
    msg!("Nb of days behind {}", nb_days_to_claim);
    nb_days_to_claim = std::cmp::min(nb_days_to_claim, STAKE_BUFFER_LEN - 1);

    if current_time
        .checked_sub(stake_pool.header.last_crank_time)
        .ok_or(AccessError::Overflow)?
        > SECONDS_IN_DAY as i64
    {
        #[cfg(not(any(feature = "days-to-sec-10s", feature = "days-to-sec-15m")))]
        return Err(AccessError::PoolMustBeCranked.into());
    }

    // Saturating as we don't want to wrap around when there haven't been sufficient cranks
    let mut i = (stake_pool.header.current_day_idx as u64 + 1).saturating_sub(nb_days_to_claim)
        % STAKE_BUFFER_LEN;

    // Compute reward for all past days
    let mut reward: u128 = 0;
    while i != (stake_pool.header.current_day_idx as u64 + 1) % STAKE_BUFFER_LEN {
        let curr_day_reward = if staker {
            stake_pool.balances[i as usize].stakers_reward
        } else {
            stake_pool.balances[i as usize].pool_reward
        };
        reward = reward
            .checked_add(curr_day_reward)
            .ok_or(AccessError::Overflow)?;
        i = (i + 1) % STAKE_BUFFER_LEN;
    }

    if reward == 0 {
        msg!("No rewards to claim, no operation.");
        return Err(AccessError::NoOp.into());
    }

    Ok(reward)
}

pub fn check_account_key(account: &AccountInfo, key: &Pubkey, error: AccessError) -> ProgramResult {
    if account.key != key {
        return Err(error.into());
    }
    Ok(())
}

pub fn check_account_owner(
    account: &AccountInfo,
    owner: &Pubkey,
    error: AccessError,
) -> ProgramResult {
    if account.owner != owner {
        return Err(error.into());
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo, error: AccessError) -> ProgramResult {
    if !(account.is_signer) {
        return Err(error.into());
    }
    Ok(())
}

pub fn assert_empty_stake_pool(stake_pool: &StakePoolRef) -> ProgramResult {
    if stake_pool.header.total_staked != 0 {
        msg!("The stake pool must be empty");
        return Err(AccessError::StakePoolMustBeEmpty.into());
    }
    Ok(())
}

pub fn assert_empty_stake_account(stake_account: &StakeAccount) -> ProgramResult {
    if stake_account.stake_amount != 0 {
        msg!("The stake account must be empty");
        return Err(AccessError::StakeAccountMustBeEmpty.into());
    }
    Ok(())
}

pub fn assert_valid_vault(account: &AccountInfo, vault_signer: &Pubkey) -> ProgramResult {
    let acc = Account::unpack(&account.data.borrow())?;
    if &acc.owner != vault_signer {
        msg!("The vault account should be owned by the stake pool signer");
        return Err(ProgramError::InvalidArgument);
    }
    if acc.close_authority.is_some() || acc.delegate.is_some() {
        msg!("Invalid vault account provided");
        return Err(ProgramError::InvalidArgument);
    }
    if acc.mint != ACCESS_MINT {
        msg!("Invalid ACCESS mint");
        #[cfg(not(feature = "no-mint-check"))]
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn assert_uninitialized(account: &AccountInfo) -> ProgramResult {
    if !account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

pub fn assert_authorized_seller(seller: &AccountInfo, seller_index: usize) -> ProgramResult {
    let expected_seller = AUTHORIZED_BOND_SELLERS
        .get(seller_index)
        .ok_or(AccessError::UnauthorizedSeller)?;
    if seller.key != expected_seller {
        return Err(AccessError::UnauthorizedSeller.into());
    }
    Ok(())
}

pub fn assert_bond_derivation(
    account: &AccountInfo,
    owner: &Pubkey,
    total_amount_sold: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let (key, _nonce) = BondAccount::create_key(owner, total_amount_sold, program_id);
    check_account_key(account, &key, AccessError::AccountNotDeterministic)?;
    Ok(())
}

pub fn assert_valid_fee(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    check_account_owner(account, &spl_token::ID, AccessError::WrongOwner)?;
    let acc = Account::unpack(&account.data.borrow())?;
    if &acc.owner != owner {
        msg!("Invalid fee account owner");
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}

pub fn calc_reward_fp32_m(
    current_time: i64,
    last_claimed_time: i64,
    stake_pool_header: crate::state::StakePoolHeader,
    balances: &Vec<RewardsTuple>,
    staker: bool,
) -> Result<u128, ProgramError> {
    let mut nb_days_to_claim =
        current_time.saturating_sub(last_claimed_time) as u64 / SECONDS_IN_DAY;
    msg!("Nb of days behind {}", nb_days_to_claim);
    nb_days_to_claim = std::cmp::min(nb_days_to_claim, STAKE_BUFFER_LEN - 1);

    if current_time
        .checked_sub(stake_pool_header.last_crank_time)
        .ok_or(AccessError::Overflow)?
        > SECONDS_IN_DAY as i64
    {
        #[cfg(not(any(feature = "days-to-sec-10s", feature = "days-to-sec-15m")))]
        return Err(AccessError::PoolMustBeCranked.into());
    }

    // Saturating as we don't want to wrap around when there haven't been sufficient cranks
    let mut i = (stake_pool_header.current_day_idx as u64 + 1).saturating_sub(nb_days_to_claim)
        % STAKE_BUFFER_LEN;

    // Compute reward for all past days
    let mut reward: u128 = 0;
    while i != (stake_pool_header.current_day_idx as u64 + 1) % STAKE_BUFFER_LEN {
        let curr_day_reward = if staker {
            balances[i as usize].stakers_reward
        } else {
            balances[i as usize].pool_reward
        };
        reward = reward
            .checked_add(curr_day_reward)
            .ok_or(AccessError::Overflow)?;
        i = (i + 1) % STAKE_BUFFER_LEN;
    }

    if reward == 0 {
        msg!("No rewards to claim, no operation.");
        return Err(AccessError::NoOp.into());
    }

    Ok(reward)
}

#[test]
pub fn test_reward() {
    let header = StakePoolHeader {
        tag: Tag::InactiveStakePool as u8,
        total_staked: 0,
        current_day_idx: 4,
        _padding: [0; 4],
        last_crank_time: 5 * SECONDS_IN_DAY as i64,
        last_claimed_time: 0,
        owner: Pubkey::default().to_bytes(),
        nonce: 0,
        vault: Pubkey::default().to_bytes(),
        minimum_stake_amount: 0,
        stakers_part: crate::state::STAKER_MULTIPLIER,
        unstake_period: crate::state::UNSTAKE_PERIOD,
    };
    let balances = vec![
        crate::state::RewardsTuple {
            pool_reward: 0,
            stakers_reward: 1,
        };
        5
    ];
    let a_reward = calc_reward_fp32_m(4 * SECONDS_IN_DAY as i64, 0, header, &balances, true);
    let b_reward = calc_reward_fp32_m(1 * SECONDS_IN_DAY as i64, 0, header, &balances, true);

    println!("Reward A {:?}", a_reward);
    println!("Reward B {:?}", b_reward);
}
