

use solana_sdk::signer::{Signer};

use solana_test_framework::*;


use crate::common::test_runner::TestRunner;


pub mod common;

#[tokio::test]
async fn common_stake_limit() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new().await.unwrap();

    // Create users
    let stake_pool_owner = tr.create_ata_account().await.unwrap();
    let staker = tr.create_ata_account().await.unwrap();

    // Mint
    tr.mint(&staker.pubkey(), 10_200).await.unwrap();

    // Create stake pool on day 1 12:00
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 1000).await.unwrap();

    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey()).await.unwrap();

    // try staking to pool 1 under the stake limit
    let result = tr.stake(&stake_pool_owner.pubkey(), &staker, 999).await;
    assert!(result.is_err());

    // try staking to pool 1 on the stake limit
    tr.stake(&stake_pool_owner.pubkey(), &staker, 1000).await.unwrap();

    // unstake
    tr.unstake(&stake_pool_owner.pubkey(), &staker, 1000).await.unwrap();

    // Create bond account
    tr.create_bond(&stake_pool_owner.pubkey(), &staker.pubkey(), 10_000, 1).await.unwrap();

    // Claim bond
    tr.claim_bond(&stake_pool_owner.pubkey(), &staker).await.unwrap();

    // staking under the stake limit should work
    tr.stake(&stake_pool_owner.pubkey(), &staker, 1).await.unwrap();
}