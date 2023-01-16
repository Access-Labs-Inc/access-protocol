use solana_test_framework::*;
use solana_sdk::signer::{Signer};
pub mod common;

use crate::common::test_runner::{TestRunner};

#[tokio::test]
async fn common_unstake_limit() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000_000).await.unwrap();

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

    // Stake to pool 1 on the stake limit
    tr.stake(&stake_pool_owner.pubkey(), &staker, 1100).await.unwrap();

    // unstake under the pool minimum should fail
    let result = tr.unstake(&stake_pool_owner.pubkey(), &staker, 200).await;
    assert!(result.is_err());
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.total_pool_staked, 1100);

    // todo investigate why this is needed and if we can get rid of it
    tr.sleep(1).await.unwrap();

    // unstake above the pool minimum should work
    tr.unstake(&stake_pool_owner.pubkey(), &staker, 100).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.total_pool_staked, 1000);

    // todo investigate why this is needed and if we can get rid of it
    tr.sleep(1).await.unwrap();

    // full unstake should work
    tr.unstake(&stake_pool_owner.pubkey(), &staker, 1000).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.total_pool_staked, 0);

    // change the pool minimum
    tr.change_pool_minimum(&stake_pool_owner, 9000).await.unwrap();

    // try staking under the pool minimum, but above the staker minimum
    let result = tr.stake(&stake_pool_owner.pubkey(), &staker, 8999).await;
    assert!(result.is_err());

    // stake above the pool minimum should work
    tr.stake(&stake_pool_owner.pubkey(), &staker, 9000).await.unwrap();

    // Create bond account
    tr.create_bond(&stake_pool_owner.pubkey(), &staker.pubkey(), 5_000, 1, 1, 1).await.unwrap();

    // Claim bond
    tr.claim_bond(&stake_pool_owner.pubkey(), &staker.pubkey()).await.unwrap();

    // unstake under the common pool minimum should fail
    let result = tr.unstake(&stake_pool_owner.pubkey(), &staker, 5001).await;
    assert!(result.is_err());

    // unstake above the common pool minimum should work
    tr.unstake(&stake_pool_owner.pubkey(), &staker, 5000).await.unwrap();

    // full unstake should still be possible
    tr.unstake(&stake_pool_owner.pubkey(), &staker, 4000).await.unwrap();

}