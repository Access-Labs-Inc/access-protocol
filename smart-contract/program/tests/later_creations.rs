use solana_sdk::signer::Signer;
use solana_test_framework::*;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn later_pool_creation() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();

    // Create users
    let stake_pool_owner = tr.create_ata_account().await.unwrap();
    let staker = tr.create_ata_account().await.unwrap();

    // Mint
    tr.mint(&staker.pubkey(), 20_400).await.unwrap();

    // Create stake pool on day 1
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 1000)
        .await
        .unwrap();
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // Stake to pool 1
    let token_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, token_amount)
        .await
        .unwrap();
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.total_staked, token_amount);

    // Crank 10 times
    for _ in 0..10 {
        tr.sleep(86400).await.unwrap();
        tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();
    }

    // Create a second pool
    let stake_pool_owner2 = tr.create_ata_account().await.unwrap();
    tr.create_stake_pool(&stake_pool_owner2.pubkey(), 1000)
        .await
        .unwrap();
    tr.activate_stake_pool(&stake_pool_owner2.pubkey())
        .await
        .unwrap();

    // Create stake account
    tr.create_stake_account(&stake_pool_owner2.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // Stake to pool 2
    let token_amount = 10_000;
    tr.stake(&stake_pool_owner2.pubkey(), &staker, token_amount)
        .await
        .unwrap();

    // Crank 10 times
    for _ in 0..10 {
        tr.sleep(86400).await.unwrap();
        tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();
        tr.crank_pool(&stake_pool_owner2.pubkey()).await.unwrap();
    }

    // Create a second staker
    let staker2 = tr.create_ata_account().await.unwrap();
    tr.mint(&staker2.pubkey(), 20_400).await.unwrap();

    // Create stake account
    tr.create_stake_account(&stake_pool_owner2.pubkey(), &staker2.pubkey())
        .await
        .unwrap();

    // Try to claim rewards as staker2 from pool2
    tr.stake(&stake_pool_owner2.pubkey(), &staker2, 10_000)
        .await
        .unwrap();
    tr.claim_staker_rewards(&stake_pool_owner2.pubkey(), &staker2)
        .await
        .unwrap();

    // Check balance
    let staker_stats = tr.staker_stats(staker2.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 10_200);
}
