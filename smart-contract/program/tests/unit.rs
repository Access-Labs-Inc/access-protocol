use solana_program::pubkey::Pubkey;
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_test_framework::*;

use access_protocol::state::Tag;

use crate::common::test_runner::TestRunner;

pub mod common;

mod basic_functionality {
    use super::*;

    #[tokio::test]
    async fn change_inflation() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Set daily inflation
        tr.change_inflation(200_000_000_000).await.unwrap();
        // Check the inflation
        let stats = tr.central_state_stats().await.unwrap();
        assert_eq!(stats.daily_inflation, 200_000_000_000);
    }

    #[tokio::test]
    async fn change_authority() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Change the authority
        let new_authority = Keypair::new();
        let stats = tr.central_state_stats().await.unwrap();
        println!("old authority: {:?}", stats.authority);
        tr.change_central_state_authority(&new_authority).await.unwrap();
        // Check the authority
        let stats = tr.central_state_stats().await.unwrap();
        assert_eq!(stats.authority, new_authority.pubkey());
    }
}

mod pool_creation_and_activation {
    use super::*;

    #[tokio::test]
    async fn create_and_activate_pool() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_ata_account().await.unwrap();
        // Create stake pool on day 1 12:00
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000).await.unwrap();
        // Check the pool
        let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
        assert_eq!(Pubkey::new(&stats.header.owner).to_string(), stake_pool_owner.pubkey().to_string());
        assert_eq!(stats.header.tag, Tag::InactiveStakePool as u8);
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
        // Check the pool
        let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
        assert_eq!(Pubkey::new(&stats.header.owner).to_string(), stake_pool_owner.pubkey().to_string());
        assert_eq!(stats.header.tag, Tag::StakePool as u8);
    }

    #[tokio::test]
    async fn cannot_change_stakers_part_to_invalid() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_ata_account().await.unwrap();
        // Create stake pool
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000).await.unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
        // Try to change the stakers part to 0
        tr.change_pool_multiplier(&stake_pool_owner, 0).await.unwrap();
        // Try to change the stakers part to 101
        let result = tr.change_pool_multiplier(&stake_pool_owner, 10000).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cannot_be_activated_if_not_created() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_ata_account().await.unwrap();
        // Activate stake pool
        let result = tr.activate_stake_pool(&stake_pool_owner.pubkey()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cannot_be_created_twice() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_ata_account().await.unwrap();
        // Create stake pool
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000).await.unwrap();
        // Try to create stake pool again
        let result = tr.create_stake_pool(&stake_pool_owner.pubkey(), 1000).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cannot_be_activated_twice() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_ata_account().await.unwrap();
        // Create stake pool
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000).await.unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
        tr.sleep(1).await.unwrap();
        // Try to activate stake pool again
        let result = tr.activate_stake_pool(&stake_pool_owner.pubkey()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cannot_be_called_before_activation() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_ata_account().await.unwrap();
        let staker = tr.create_ata_account().await.unwrap();
         // Mint to staker
        tr.mint(&staker.pubkey(), 100_000_000_000).await.unwrap();
        // Create stake pool
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000).await.unwrap();
        // Try to stake
        let result = tr.stake(&stake_pool_owner.pubkey(), &staker,  100000).await;
        assert!(result.is_err());
        // Try to create a bond
        let result = tr.create_bond(&stake_pool_owner.pubkey(), &staker.pubkey(), 10000, 1).await;
        assert!(result.is_err());
    }
}

mod pool_settings {
    use super::*;

    #[tokio::test]
    async fn can_change_minimum_stake_amount() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_ata_account().await.unwrap();
        // Create stake pool
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000).await.unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
        // Change the minimum stake amount
        tr.change_pool_minimum(&stake_pool_owner, 1000).await.unwrap();
        // Check the pool
        let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
        assert_eq!(stats.header.minimum_stake_amount, 1000);
    }

    #[tokio::test]
    async fn can_change_stakers_part() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new().await.unwrap();
        // Create users
        let stake_pool_owner = tr.create_ata_account().await.unwrap();
        // Create stake pool
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 10000).await.unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
        // Change the stakers part
        tr.change_pool_multiplier(&stake_pool_owner, 20).await.unwrap();
        // Check the pool
        let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
        assert_eq!(stats.header.stakers_part, 20);
    }
}