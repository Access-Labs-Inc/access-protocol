use solana_sdk::signer::Signer;

use solana_test_framework::*;

pub mod common;
use crate::common::test_runner::TestRunner;

#[tokio::test]
async fn claim_before_crank() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();

    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    let stake_pool2_owner = tr.create_user_with_ata().await.unwrap();
    let staker = tr.create_user_with_ata().await.unwrap();

    // Mint
    tr.mint(&staker.pubkey(), 20_400).await.unwrap();

    // Setup stake pool on day 1 12:00
    tr.create_pool(&stake_pool_owner, 1000).await.unwrap();
    tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey()).await.unwrap();

    // Wait 1 hour
    tr.sleep(3600).await.unwrap();

    // Setup stake pool 2 on day 1 13:00
    tr.create_pool(&stake_pool2_owner, 1000).await.unwrap();
    tr.activate_stake_pool(&stake_pool2_owner.pubkey()).await.unwrap();
    tr.create_stake_account(&stake_pool2_owner.pubkey(), &staker.pubkey()).await.unwrap();

    // Stake to pool 1 and 2
    let token_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, token_amount).await.unwrap();
    tr.stake(&stake_pool2_owner.pubkey(), &staker, token_amount).await.unwrap();

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

    // Claiming staker rewards in pool2 - no rewards yet
    assert!(tr.claim_staker_rewards(&stake_pool2_owner.pubkey(), &staker)
        .await
        .is_err());

    // Stake to pool 2 should fail
    let result = tr
        .stake(&stake_pool2_owner.pubkey(), &staker, token_amount)
        .await;
    assert!(result.is_err());
    tr.sleep(1).await.unwrap();

    // Crank pool 2
    tr.crank_pool(&stake_pool2_owner.pubkey()).await.unwrap();

    // wait until day 3 12:15
    tr.sleep(86400).await.unwrap();

    // Crank pool 1 (+ implicitly the whole system)
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.current_day_idx, 1);
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.current_day_idx, 2);

    // Claim staker rewards in pool2 - should fail (one day)
    let stake_account_stats = tr.stake_account_stats(staker.pubkey(), stake_pool2_owner.pubkey()).await.unwrap();
    assert_eq!(stake_account_stats.last_claimed_offset, 0);

    tr.claim_staker_rewards(&stake_pool2_owner.pubkey(), &staker)
        .await
        .unwrap_err();

    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 250_000);
    let stake_account_stats = tr.stake_account_stats(staker.pubkey(), stake_pool2_owner.pubkey()).await.unwrap();
    assert_eq!(stake_account_stats.last_claimed_offset, 0);

    // Crank pool 2
    let pool_stats = tr.pool_stats(stake_pool2_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.current_day_idx, 1);
    tr.crank_pool(&stake_pool2_owner.pubkey()).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool2_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.current_day_idx, 2);

    // Claim staker rewards in pool2 - should succeed (another day)
    tr.sleep(1).await.unwrap();
    tr.claim_staker_rewards(&stake_pool2_owner.pubkey(), &staker)
        .await
        .unwrap();
    let stake_account_stats = tr.stake_account_stats(staker.pubkey(), stake_pool2_owner.pubkey()).await.unwrap();
    assert_eq!(stake_account_stats.last_claimed_offset, 2);
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 750_000);
}
