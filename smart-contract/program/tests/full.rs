use solana_sdk::signer::Signer;
use solana_test_framework::*;

use crate::common::test_runner::TestRunner;

pub mod common;

const DAY: u64 = 86400;

#[tokio::test]
async fn full_system_test() {
    // ------------------------------------------
    // DAY 1
    //-------------------------------------------
    // Setup the token + basic accounts
    let mut tr = TestRunner::new().await.unwrap();
    // Setup 3 pool owners
    let pool_owner = tr.create_ata_account().await.unwrap();
    let pool_owner2 = tr.create_ata_account().await.unwrap();
    let pool_owner3 = tr.create_ata_account().await.unwrap();
    // Setup 3 airdrop users
    let airdrop_user1 = tr.create_ata_account().await.unwrap();
    let airdrop_user2 = tr.create_ata_account().await.unwrap();
    let _airdrop_user3 = tr.create_ata_account().await.unwrap();
    // Setup 2 vesting users
    let _vesting_user1 = tr.create_ata_account().await.unwrap();
    let _vesting_user2 = tr.create_ata_account().await.unwrap();

    // Setup all the pools
    tr.create_stake_pool(&pool_owner.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_pool(&pool_owner2.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_pool(&pool_owner3.pubkey(), 1_000_000_000).await.unwrap();
    // Activate the pools
    tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();
    tr.activate_stake_pool(&pool_owner2.pubkey()).await.unwrap();

    // ------------------------------------------
    // DAY 2
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    // First two airdrop users get airdrop of 100000 tokens to the first pool with the release date on day 10
    tr.create_bond(&pool_owner.pubkey(), &airdrop_user1.pubkey(), 100_000_000_000, 8*DAY as i64).await.unwrap();
    tr.create_bond(&pool_owner.pubkey(), &airdrop_user2.pubkey(), 100_000_000_000, 8*DAY as i64).await.unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &airdrop_user1.pubkey()).await.unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &airdrop_user2.pubkey()).await.unwrap();

   // todo continue
}