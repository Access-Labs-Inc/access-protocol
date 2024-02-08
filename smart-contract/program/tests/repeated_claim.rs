use solana_sdk::signer::Signer;

use solana_test_framework::*;

pub mod common;
use crate::common::test_runner::TestRunner;

#[tokio::test]
async fn repeated_claim() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();

    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    let stake_pool2_owner = tr.create_user_with_ata().await.unwrap();
    let staker = tr.create_user_with_ata().await.unwrap();

    // Mint
    tr.mint(&staker.pubkey(), 10_200).await.unwrap();

    // Create stake pool on day 1 12:00
    tr.create_pool(&stake_pool_owner, 1000)
        .await
        .unwrap();

    // // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // Wait 1 hour
    tr.sleep(3600).await.unwrap();

    // Create stake pool 2 on day 1 13:00
    tr.create_pool(&stake_pool2_owner, 1000)
        .await
        .unwrap();

    // Activate stake pool 2
    tr.activate_stake_pool(&stake_pool2_owner.pubkey())
        .await
        .unwrap();

    // Create stake account 2
    tr.create_stake_account(&stake_pool2_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // Stake to pool 1
    let token_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, token_amount)
        .await
        .unwrap();

    // wait until day 2 12:15
    tr.sleep(86400 - 2700).await.unwrap();

    // Crank pool 1 (+ implicitly the whole system)
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim pool 1 rewards
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();

    // Claim staker rewards in pool 1

    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();

    // Unstake from pool 1
    tr.unstake(&stake_pool_owner.pubkey(), &staker, token_amount)
        .await
        .unwrap();

    // Stake to pool 2 should fail
    let result = tr
        .stake(&stake_pool2_owner.pubkey(), &staker, token_amount)
        .await;
    assert!(result.is_err());
    tr.sleep(1).await.unwrap();

    // Crank pool 2
    tr.crank_pool(&stake_pool2_owner.pubkey()).await.unwrap();

    // Stake to pool 2 should succeed
    tr.stake(&stake_pool2_owner.pubkey(), &staker, token_amount)
        .await
        .unwrap();
    tr.sleep(1).await.unwrap();

    // Claim stake pool rewards 2
    assert!(tr.claim_pool_rewards(&stake_pool2_owner).await.is_err());

    // Claim rewards 2
    tr.claim_staker_rewards(&stake_pool2_owner.pubkey(), &staker)
        .await
        .unwrap();

    // Check results
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 499_800);
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.balance, 500_000);
    assert_eq!(pool_stats.header.total_staked, 0);
    let pool_stats2 = tr.pool_stats(stake_pool2_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats2.balance, 0);
    assert_eq!(pool_stats2.header.total_staked, 10_000);
}
