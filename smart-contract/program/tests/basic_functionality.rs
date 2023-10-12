use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_test_framework::*;
use access_protocol::state::Tag;

use crate::common::test_runner::{INITIAL_SUPPLY, TestRunner};

pub mod common;

#[tokio::test]
async fn change_inflation() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();

    // Set daily inflation - should fail as it is over 100% per year
    tr.change_inflation(INITIAL_SUPPLY /365 + 2).await.unwrap_err();
    // Check the inflation
    let stats = tr.central_state_stats().await.unwrap();
    assert_eq!(stats.account.daily_inflation, 1_000_000);

    // increase supply
    tr.sleep(1).await.unwrap();
    // Set daily inflation - should succeed
    tr.change_inflation(INITIAL_SUPPLY / 365).await.unwrap();
    // Check the inflation
    let stats = tr.central_state_stats().await.unwrap();
    assert_eq!(stats.account.daily_inflation, INITIAL_SUPPLY / 365);
}


#[tokio::test]
async fn change_authority() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    // Change the authority
    let new_authority = Keypair::new();
    let stats = tr.central_state_stats().await.unwrap();
    println!("old authority: {:?}", stats.account.authority);
    tr.change_central_state_authority(&new_authority)
        .await
        .unwrap();
    // Check the authority
    let stats = tr.central_state_stats().await.unwrap();
    assert_eq!(stats.account.authority, new_authority.pubkey());
}

#[tokio::test]
async fn zero_inflation_start() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(0).await.unwrap();
    // Sleep for 5 days
    tr.sleep(5 * 86400).await.unwrap();
    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    // Create a pool
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000)
        .await
        .unwrap();
    // Check the pool
    let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(
        Pubkey::new(&stats.header.owner).to_string(),
        stake_pool_owner.pubkey().to_string()
    );
    assert_eq!(stats.header.tag, Tag::InactiveStakePool as u8);
    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();
    // Crank
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();
    // Check the central state
    let stats = tr.central_state_stats().await.unwrap();
    assert_eq!(stats.account.last_snapshot_offset, 5);
    assert_eq!(stats.account.total_staked, 0);
    assert_eq!(stats.account.total_staked_snapshot, 0);
}