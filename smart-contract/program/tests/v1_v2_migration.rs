use solana_program::pubkey::Pubkey;
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_test_framework::*;

use access_protocol::state::Tag;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn v1_v2_migration() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new_v1(0).await.unwrap();

    let staker = tr.create_ata_account().await.unwrap();

    // Mint - V1 should work
    tr.mint(&staker.pubkey(), 10_200_000_000).await.unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 10_200_000_000);

    // Upgrade to V2
    tr.upgrade_v2().await.unwrap();

    // now expect an error
    tr.sleep(20).await.unwrap(); // so that we are not sending the same transaction twice in the same block
    tr.mint(&staker.pubkey(), 10_200_000_000).await.unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    // assert_eq!(staker_stats.balance, 10_200_000_000);
}