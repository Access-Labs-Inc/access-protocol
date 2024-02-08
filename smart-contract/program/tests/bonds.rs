use solana_sdk::signer::Signer;
use solana_test_framework::*;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn permissionless_claim() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    let _staker = tr.create_user_with_ata().await.unwrap();
    // Create stake pool
    tr.create_pool(&stake_pool_owner, 10000)
        .await
        .unwrap();
    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();
    // Create bond
    // tr.create_bond(&stake_pool_owner.pubkey(), &staker.pubkey(), 10000, 1).await.unwrap();
    // // Claim bond
    // tr.claim_bond(&stake_pool_owner.pubkey(), &staker.pubkey()).await.unwrap();
}

#[tokio::test]
async fn signed_claim() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    let staker = tr.create_user_with_ata().await.unwrap();
    // Mint to staker
    tr.mint(&staker.pubkey(), 100_000_000_000).await.unwrap();
    // Create stake pool
    tr.create_pool(&stake_pool_owner, 10000)
        .await
        .unwrap();
    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();
    // Create real bond with quote amount
    tr.create_bond_with_quote(&stake_pool_owner.pubkey(), &staker.pubkey(), 10000, 200, 1)
        .await
        .unwrap();
    // Claim bond without signature should fail
    assert!(tr
        .claim_bond(&stake_pool_owner.pubkey(), &staker.pubkey())
        .await
        .is_err());
    // Claim bond with signature should succeed
    tr.claim_bond_with_quote(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
}