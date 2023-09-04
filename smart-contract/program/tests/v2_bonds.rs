use solana_sdk::signer::Signer;
use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn signed_claim() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    // Create users
    let stake_pool_owner = tr.create_ata_account().await.unwrap();
    let from = tr.create_ata_account().await.unwrap();
    let to = tr.create_ata_account().await.unwrap();
    // Mint to staker
    tr.mint(&from.pubkey(), 100_000_000_000).await.unwrap();
    // Create stake pool
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000).await.unwrap();
    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
    // Create real bond with quote amount
    tr.create_bond_v2(
        &from,
        &to.pubkey(),
        &stake_pool_owner.pubkey(),
        10000,
        200,
    ).await.unwrap();
}