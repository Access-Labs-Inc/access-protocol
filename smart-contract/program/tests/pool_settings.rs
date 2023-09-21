use solana_sdk::signer::Signer;
use solana_test_framework::*;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn can_change_minimum_stake_amount() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    // Create stake pool
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000)
        .await
        .unwrap();
    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();
    // Change the minimum stake amount
    tr.change_pool_minimum(&stake_pool_owner, 1000)
        .await
        .unwrap();
    // Check the pool
    let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stats.header.minimum_stake_amount, 1000);
}

#[tokio::test]
async fn can_change_stakers_part() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    // Create stake pool
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000)
        .await
        .unwrap();
    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();
    // Change the stakers part
    tr.change_pool_multiplier(&stake_pool_owner, 20)
        .await
        .unwrap();
    // Check the pool
    let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stats.header.stakers_part, 20);
}