use solana_program::pubkey::Pubkey;
use solana_sdk::signer::{Signer};

use solana_test_framework::*;
use access_protocol::instruction::create_bond;
use access_protocol::state::Tag;


use crate::common::test_runner::TestRunner;


pub mod common;

const hour_seconds: u64 = 3600;

#[tokio::test]
async fn end_to_end() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new().await.unwrap();

    let cs_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(cs_stats.tag, Tag::CentralState);
    assert_eq!(cs_stats.daily_inflation, 1_000_000);
    assert_eq!(cs_stats.token_mint.to_string(), tr.get_mint().to_string());
    assert_eq!(cs_stats.authority, tr.get_authority());
    assert_eq!(cs_stats.creation_time, tr.get_current_time().await);
    assert_eq!(cs_stats.total_staked, 0);
    assert_eq!(cs_stats.total_staked_snapshot, 0);
    assert_eq!(cs_stats.last_snapshot_offset, 0);

    // todo Edit metadata
    // ...

    // Create users
    let stake_pool_owner = tr.create_ata_account().await.unwrap();
    let staker = tr.create_ata_account().await.unwrap();



    // Create stake pool
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 1000).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.tag, Tag::InactiveStakePool as u8);
    assert_eq!(pool_stats.header.current_day_idx, 0);
    assert_eq!(pool_stats.header.minimum_stake_amount, 1000);
    assert_eq!(pool_stats.header.total_staked, 0);
    assert_eq!(pool_stats.header.total_staked_delta, 0);
    assert_eq!(pool_stats.header.last_delta_update_offset, 0);
    assert_eq!(pool_stats.header.last_claimed_offset, 0);
    assert_eq!(pool_stats.header.stakers_part, 50);
    assert_eq!(Pubkey::new(&pool_stats.header.owner).to_string(), stake_pool_owner.pubkey().to_string());

    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.tag, Tag::StakePool as u8);

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey()).await.unwrap();
    let stake_account_stats = tr.stake_account_stats(staker.pubkey(), stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_account_stats.tag, Tag::StakeAccount);
    assert_eq!(stake_account_stats.owner, staker.pubkey());
    assert_eq!(stake_account_stats.stake_amount, 0);
    let stake_pool_pda_key = tr.get_pool_pda(&stake_pool_owner.pubkey());
    assert_eq!(stake_account_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(stake_account_stats.last_claimed_offset, 0);
    assert_eq!(stake_account_stats.pool_minimum_at_creation, 1000);

    // Create a bond
    let bond_amount = 5_000_000;
    let unlock_after = 10;
    tr.create_bond(&stake_pool_owner.pubkey(), &staker.pubkey(), bond_amount, unlock_after).await.unwrap();
    let bond_stats = tr.bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount).await.unwrap();
    assert_eq!(bond_stats.tag, Tag::InactiveBondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, bond_amount);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(bond_stats.unlock_start_date, tr.get_current_time().await + unlock_after);
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    assert_eq!(bond_stats.last_unlock_time, tr.get_current_time().await + unlock_after);
    assert_eq!(bond_stats.total_unlocked_amount, 0);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 0);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());

    // Claim bond
    tr.claim_bond(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let bond_stats = tr.bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount).await.unwrap();
    let bond_creation_time = tr.get_current_time().await;
    assert_eq!(bond_stats.tag, Tag::BondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, bond_amount);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(bond_stats.quote_mint, tr.get_mint());
    assert_eq!(bond_stats.seller_token_account, tr.get_bond_seller_ata());
    assert_eq!(bond_stats.unlock_start_date, bond_creation_time + unlock_after);
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    assert_eq!(bond_stats.last_unlock_time, bond_creation_time + unlock_after);
    assert_eq!(bond_stats.total_unlocked_amount, 0);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 0);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());

    // Unlock bond
    let sleep_time = (unlock_after as f32 * 1.5) as u64;
    tr.sleep(sleep_time).await.unwrap();

    tr.unlock_bond(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let bond_stats = tr.bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount).await.unwrap();
    assert_eq!(bond_stats.tag, Tag::BondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, 0);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(bond_stats.quote_mint, tr.get_mint());
    assert_eq!(bond_stats.seller_token_account, tr.get_bond_seller_ata());
    assert_eq!(bond_stats.unlock_start_date, bond_creation_time + unlock_after);
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    assert_eq!(bond_stats.last_unlock_time, tr.get_current_time().await);
    assert_eq!(bond_stats.total_unlocked_amount, bond_amount);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 0);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());

    // Stake
    let stake_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, stake_amount).await.unwrap();

    tr.sleep(5).await.unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 4989800);
    let stake_account_stats = tr.stake_account_stats(staker.pubkey(), stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_account_stats.tag, Tag::StakeAccount);
    assert_eq!(stake_account_stats.owner, staker.pubkey());
    assert_eq!(stake_account_stats.stake_amount, stake_amount);
    assert_eq!(stake_account_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(stake_account_stats.last_claimed_offset, 1);
    assert_eq!(stake_account_stats.pool_minimum_at_creation, 1000);

    // Crank
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();
    let stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_pool_stats.header.tag, Tag::StakePool as u8);
    assert_eq!(stake_pool_stats.header.current_day_idx, 2);
    assert_eq!(stake_pool_stats.header.minimum_stake_amount, 1000);
    assert_eq!(stake_pool_stats.header.total_staked, stake_amount);
    assert_eq!(stake_pool_stats.header.total_staked_delta, 0);
    assert_eq!(stake_pool_stats.header.last_delta_update_offset, 2);
    assert_eq!(stake_pool_stats.header.last_claimed_offset, 0);
    assert_eq!(stake_pool_stats.header.stakers_part, 50);
    assert_eq!(Pubkey::new(&pool_stats.header.owner).to_string(), stake_pool_owner.pubkey().to_string());

    // Claim bond rewards
    tr.claim_bond_rewards(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let bond_stats = tr.bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount).await.unwrap();
    assert_eq!(bond_stats.tag, Tag::BondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, 0);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(bond_stats.quote_mint, tr.get_mint());
    assert_eq!(bond_stats.seller_token_account, tr.get_bond_seller_ata());
    assert_eq!(bond_stats.unlock_start_date, bond_creation_time + unlock_after);
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    // assert_eq!(bond_stats.last_unlock_time, bond_creation_time + unlock_after);
    assert_eq!(bond_stats.total_unlocked_amount, bond_amount);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 2);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());

    return;
    // Stake to pool 1
    let token_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, token_amount).await.unwrap();
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.total_staked, token_amount);

    // Claim bond
    tr.claim_bond(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.total_staked, 20_000);

    // wait until day 2 12:00
    tr.sleep(86400).await.unwrap();

    // Crank pool 1 (+ implicitly the whole system)
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim pool 1 rewards
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.balance, 500_000);

    // Claim staker rewards in pool 1
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 250_000);

    // Claim bond rewards
    tr.claim_bond_rewards(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 500_000);

    // 23 hours later
    tr.sleep(82800).await.unwrap();

    // Crank should fail
    let crank_result = tr.crank_pool(&stake_pool_owner.pubkey()).await;
    assert!(crank_result.is_err());

    // Try to claim rewards again
    let result = tr.claim_bond_rewards(&stake_pool_owner.pubkey(), &staker).await;
    assert!(result.is_err());
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let result = tr.claim_pool_rewards(&stake_pool_owner).await;
    assert!(result.is_err());

    // check balances
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 500_000);
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.balance, 500_000);

    // 1 hour later
    tr.sleep(3600).await.unwrap();

    // Crank should succeed
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim rewards again
    tr.claim_bond_rewards(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 750_000);
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker).await.unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 1_000_000);
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.balance, 1_000_000);
}