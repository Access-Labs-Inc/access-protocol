use solana_sdk::signer::Signer;
use solana_test_framework::*;
use crate::common::test_runner::TestRunner;

pub mod common;

    use solana_program::pubkey::Pubkey;
    use access_protocol::state::Tag;

    #[tokio::test]
    async fn create_and_activate_pool() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(1_000_000).await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
        // Create stake pool on day 1 12:00
        tr.create_pool(&stake_pool_owner, 10000)
            .await
            .unwrap();
        // Check the pool
        let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
        assert_eq!(
            Pubkey::from(stats.header.owner).to_string(),
            stake_pool_owner.pubkey().to_string()
        );
        assert_eq!(stats.header.tag, Tag::InactiveStakePool as u8);
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey())
            .await
            .unwrap();
        // Check the pool
        let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
        assert_eq!(
            Pubkey::from(stats.header.owner).to_string(),
            stake_pool_owner.pubkey().to_string()
        );
        assert_eq!(stats.header.tag, Tag::StakePool as u8);
    }

    #[tokio::test]
    async fn cannot_change_stakers_part_to_invalid() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(1_000_000).await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
        // Create stake pool
        tr.create_pool(&stake_pool_owner, 10000)
            .await
            .unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey())
            .await
            .unwrap();
        // Try to change the stakers part to 0
        tr.change_pool_multiplier(&stake_pool_owner, 0)
            .await
            .unwrap();
        // Try to change the stakers part to 101
        let result = tr.change_pool_multiplier(&stake_pool_owner, 10000).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cannot_be_activated_if_not_created() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(1_000_000).await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
        // Activate stake pool
        let result = tr.activate_stake_pool(&stake_pool_owner.pubkey()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cannot_be_created_twice() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(1_000_000).await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
        // Create stake pool
        tr.create_pool(&stake_pool_owner, 10000)
            .await
            .unwrap();
        // Try to create stake pool again
        let result = tr.create_pool(&stake_pool_owner, 1000).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cannot_be_activated_twice() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(1_000_000).await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
        // Create stake pool
        tr.create_pool(&stake_pool_owner, 10000)
            .await
            .unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey())
            .await
            .unwrap();
        tr.sleep(1).await.unwrap();
        // Try to activate stake pool again
        let result = tr.activate_stake_pool(&stake_pool_owner.pubkey()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cannot_be_called_before_activation() {
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
        // Try to stake
        let result = tr.stake(&stake_pool_owner.pubkey(), &staker, 100000).await;
        assert!(result.is_err());
        // Try to create a bond
        let result = tr
            .create_bond(&stake_pool_owner.pubkey(), &staker.pubkey(), 10000, 1, 1, 1)
            .await;
        assert!(result.is_err());
    }
