use solana_sdk::signer::Signer;
use solana_test_framework::*;

use access_protocol::state::STAKE_BUFFER_LEN;

use crate::common::test_runner::TestRunner;

pub mod common;

const DAY: u64 = 86400;
const DAILY_INFLATION: u64 = 10_000_000_000_000_000; // 1M of tokens

mod rewards_wrap {
    use super::*;

    #[tokio::test]
    async fn rewards_long_time() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(DAILY_INFLATION).await.unwrap();

        // Setup 2 pool owners
        let pool_owner = tr.create_ata_account().await.unwrap();
        let pool_owner2 = tr.create_ata_account().await.unwrap();

        // Setup all the pools
        tr.create_stake_pool(&pool_owner.pubkey(), 1_000_000_000).await.unwrap();
        tr.create_stake_pool(&pool_owner2.pubkey(), 1_000_000_000).await.unwrap();

        // Activate the pools
        tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();
        tr.activate_stake_pool(&pool_owner2.pubkey()).await.unwrap();

        // Setup 1 staker with 5_100_000_000 tokens in his account and appropriate stake accounts pool 1
        let staker = tr.create_ata_account().await.unwrap();
        tr.mint(&staker.pubkey(), 5_100_000_000).await.unwrap();
        tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey()).await.unwrap();

        // Stake 5_000_000_000 tokens to pool 1 and 5_000_000_000 tokens to pool 2
        tr.stake(&pool_owner.pubkey(), &staker, 5_000_000_000).await.unwrap();

        // airdrop 5_000_000_000 tokens to staker to pool 2
        tr.create_bond(&pool_owner2.pubkey(), &staker.pubkey(), 5_000_000_000, 1, 8 * DAY as i64, 1).await.unwrap();
        tr.claim_bond(&pool_owner2.pubkey(), &staker.pubkey()).await.unwrap();

        // crank STAKE_BUFFER_LEN times
        for _ in 0..2 * STAKE_BUFFER_LEN + 3 {
            tr.sleep(DAY).await.unwrap();
            tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
            tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
        }

        // Claim pool 1 rewards
        tr.claim_pool_rewards(&pool_owner).await.unwrap();
        let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
        assert_eq!(pool_stats.balance, STAKE_BUFFER_LEN * DAILY_INFLATION / 4);

        // Claim staker rewards
        tr.claim_staker_rewards(&pool_owner.pubkey(), &staker).await.unwrap();
        let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
        assert_eq!(staker_stats.balance, STAKE_BUFFER_LEN * DAILY_INFLATION / 4);

        // Claim pool 2 rewards
        tr.claim_pool_rewards(&pool_owner2).await.unwrap();
        let pool_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
        assert_eq!(pool_stats.balance, STAKE_BUFFER_LEN * DAILY_INFLATION / 4);

        // Claim bond rewards
        tr.claim_bond_rewards(&pool_owner2.pubkey(), &staker).await.unwrap();
        let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
        assert_eq!(staker_stats.balance, STAKE_BUFFER_LEN * DAILY_INFLATION / 2);
    }

    #[tokio::test]
    async fn rewards_on_bound() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(DAILY_INFLATION).await.unwrap();

        // Setup 2 pool owners
        let pool_owner = tr.create_ata_account().await.unwrap();
        let pool_owner2 = tr.create_ata_account().await.unwrap();

        // Setup all the pools
        tr.create_stake_pool(&pool_owner.pubkey(), 1_000_000_000).await.unwrap();
        tr.create_stake_pool(&pool_owner2.pubkey(), 1_000_000_000).await.unwrap();

        // Activate the pools
        tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();
        tr.activate_stake_pool(&pool_owner2.pubkey()).await.unwrap();

        // Setup 1 staker with 5_100_000_000 tokens in his account and appropriate stake accounts pool 1
        let staker = tr.create_ata_account().await.unwrap();
        tr.mint(&staker.pubkey(), 5_100_000_000).await.unwrap();
        tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey()).await.unwrap();

        // Stake 5_000_000_000 tokens to pool 1 and 5_000_000_000 tokens to pool 2
        tr.stake(&pool_owner.pubkey(), &staker, 5_000_000_000).await.unwrap();

        // airdrop 5_000_000_000 tokens to staker to pool 2
        tr.create_bond(&pool_owner2.pubkey(), &staker.pubkey(), 5_000_000_000, 1, 8 * DAY as i64, 1).await.unwrap();
        tr.claim_bond(&pool_owner2.pubkey(), &staker.pubkey()).await.unwrap();

        // crank STAKE_BUFFER_LEN times
        for i in 0..STAKE_BUFFER_LEN + 10 {
            tr.sleep(DAY).await.unwrap();
            tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
            tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();

            // Claim pool 1 rewards
            tr.claim_pool_rewards(&pool_owner).await.unwrap();
            let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
            assert_eq!(pool_stats.balance, (i + 1) * DAILY_INFLATION / 4);

            // Claim staker rewards
            tr.claim_staker_rewards(&pool_owner.pubkey(), &staker).await.unwrap();
            let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
            assert_eq!(staker_stats.balance, i * DAILY_INFLATION / 2 + DAILY_INFLATION / 4);

            // Claim pool 2 rewards
            tr.claim_pool_rewards(&pool_owner2).await.unwrap();
            let pool_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
            assert_eq!(pool_stats.balance, (i + 1) * DAILY_INFLATION / 4);

            // Claim bond rewards
            tr.claim_bond_rewards(&pool_owner2.pubkey(), &staker).await.unwrap();
            let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
            assert_eq!(staker_stats.balance, (i + 1) * DAILY_INFLATION / 2);
        }
    }

    #[tokio::test]
    async fn rewards_over_bound() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(DAILY_INFLATION).await.unwrap();

        // Setup 2 pool owners
        let pool_owner = tr.create_ata_account().await.unwrap();
        let pool_owner2 = tr.create_ata_account().await.unwrap();

        // Setup all the pools
        tr.create_stake_pool(&pool_owner.pubkey(), 1_000_000_000).await.unwrap();
        tr.create_stake_pool(&pool_owner2.pubkey(), 1_000_000_000).await.unwrap();

        // Activate the pools
        tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();
        tr.activate_stake_pool(&pool_owner2.pubkey()).await.unwrap();

        // Setup 1 staker with 5_100_000_000 tokens in his account and appropriate stake accounts pool 1
        let staker = tr.create_ata_account().await.unwrap();
        tr.mint(&staker.pubkey(), 5_100_000_000).await.unwrap();
        tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey()).await.unwrap();

        // Stake 5_000_000_000 tokens to pool 1 and 5_000_000_000 tokens to pool 2
        tr.stake(&pool_owner.pubkey(), &staker, 5_000_000_000).await.unwrap();

        // airdrop 5_000_000_000 tokens to staker to pool 2
        tr.create_bond(&pool_owner2.pubkey(), &staker.pubkey(), 5_000_000_000, 1, 8 * DAY as i64, 1).await.unwrap();
        tr.claim_bond(&pool_owner2.pubkey(), &staker.pubkey()).await.unwrap();

        // crank STAKE_BUFFER_LEN times
        for i in 0..STAKE_BUFFER_LEN - 5 {
            tr.sleep(DAY).await.unwrap();
            tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
            tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();

            // Claim pool 1 rewards
            tr.claim_pool_rewards(&pool_owner).await.unwrap();
            let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
            assert_eq!(pool_stats.balance, (i + 1) * DAILY_INFLATION / 4);

            // Claim staker rewards
            tr.claim_staker_rewards(&pool_owner.pubkey(), &staker).await.unwrap();
            let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
            assert_eq!(staker_stats.balance, i * DAILY_INFLATION / 2 + DAILY_INFLATION / 4);

            // Claim pool 2 rewards
            tr.claim_pool_rewards(&pool_owner2).await.unwrap();
            let pool_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
            assert_eq!(pool_stats.balance, (i + 1) * DAILY_INFLATION / 4);

            // Claim bond rewards
            tr.claim_bond_rewards(&pool_owner2.pubkey(), &staker).await.unwrap();
            let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
            assert_eq!(staker_stats.balance, (i + 1) * DAILY_INFLATION / 2);
        }

        // crank 10 more times without claim
        for _ in 0..10 {
            tr.sleep(DAY).await.unwrap();
            tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
            tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
        }

        // Claim pool 1 rewards
        tr.claim_pool_rewards(&pool_owner).await.unwrap();
        let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
        assert_eq!(pool_stats.balance, (STAKE_BUFFER_LEN + 5) * DAILY_INFLATION / 4);

        // Claim staker rewards
        tr.claim_staker_rewards(&pool_owner.pubkey(), &staker).await.unwrap();
        let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
        assert_eq!(staker_stats.balance, (STAKE_BUFFER_LEN - 5) * DAILY_INFLATION / 2 + 10 * DAILY_INFLATION / 4);

        // Claim pool 2 rewards
        tr.claim_pool_rewards(&pool_owner2).await.unwrap();
        let pool_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
        assert_eq!(pool_stats.balance, (STAKE_BUFFER_LEN + 5) * DAILY_INFLATION / 4);

        // Claim bond rewards
        tr.claim_bond_rewards(&pool_owner2.pubkey(), &staker).await.unwrap();
        let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
        assert_eq!(staker_stats.balance, (STAKE_BUFFER_LEN + 5) * DAILY_INFLATION / 2);
    }
}