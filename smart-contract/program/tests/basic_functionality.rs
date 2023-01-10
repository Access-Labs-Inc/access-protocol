use solana_test_framework::*;
use std::error::Error;

use borsh::BorshDeserialize;
use mpl_token_metadata::pda::find_metadata_account;
use solana_program::{pubkey::Pubkey, system_program};
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

use solana_program_test::{processor, ProgramTest};
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::sysvar::clock;
use solana_test_framework::*;
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};

use access_protocol::{
    entrypoint::process_instruction,
    instruction::{
        activate_stake_pool, admin_mint,
        claim_pool_rewards, claim_rewards,
        crank, create_central_state, create_stake_account,
        create_stake_pool, stake, unstake,
    },
};
use access_protocol::instruction::{change_central_state_authority, change_inflation, change_pool_minimum, change_pool_multiplier, claim_bond, claim_bond_rewards, create_bond, unlock_bond_tokens};
use access_protocol::state::{BondAccount, CentralState, StakeAccount, StakePoolHeader, Tag};


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
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 101).await.unwrap();
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
        tr.create_stake_pool(&stake_pool_owner.pubkey(), 101).await.unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
        // Try to change the stakers part to 0
        tr.change_pool_multiplier(&stake_pool_owner, 0).await.unwrap();
        // Try to change the stakers part to 101
        let result = tr.change_pool_multiplier(&stake_pool_owner, 101).await;
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
}