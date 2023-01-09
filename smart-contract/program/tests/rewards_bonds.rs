use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::sysvar::clock;
use solana_test_framework::*;
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use std::os::macos::raw::stat;
pub mod common;
use crate::common::test_runner::TestRunner;
use crate::common::utils::{mint_bootstrap, sign_send_instructions};
use access_protocol::instruction::create_bond;
use access_protocol::{
    entrypoint::process_instruction,
    instruction::{
        activate_stake_pool, admin_mint, claim_pool_rewards, claim_rewards, crank,
        create_central_state, create_stake_account, create_stake_pool, execute_unstake, stake,
        unstake,
    },
};
use mpl_token_metadata::pda::find_metadata_account;

#[tokio::test]
async fn rewards_bonds() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new().await.unwrap();

    // Create users
    let stake_pool_owner = tr.create_ata_account().await.unwrap();
    let staker = tr.create_ata_account().await.unwrap();

    // Mint
    tr.mint(&staker.pubkey(), 10_200).await.unwrap();

    // Create stake pool on day 1 12:00
    tr.create_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();

    // // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // Stake to pool 1
    let token_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, token_amount)
        .await
        .unwrap();
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.total_staked, token_amount);

    // Create bond account
    tr.create_bond(&stake_pool_owner.pubkey(), &staker.pubkey(), 10_000)
        .await
        .unwrap();
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.total_staked, token_amount);

    // Claim bond
    tr.claim_bond(&stake_pool_owner.pubkey(), &staker, 10_000)
        .await
        .unwrap();
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.total_staked, 20_000);

    // wait until day 2 12:00
    tr.sleep(86400).await.unwrap();

    // Crank pool 1 (+ implicitly the whole system)
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim pool 1 rewards
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.balance, 500_000);

    // Claim staker rewards in pool 1
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 250_000);

    // Claim bond rewards
    tr.claim_bond_rewards(&stake_pool_owner.pubkey(), &staker, 10_000)
        .await
        .unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 500_000);

    // 23 hours later
    tr.sleep(82800).await.unwrap();

    // Crank should fail
    let crank_result = tr.crank_pool(&stake_pool_owner.pubkey()).await;
    assert!(crank_result.is_err());

    // Try to claim rewards again
    let result = tr
        .claim_bond_rewards(&stake_pool_owner.pubkey(), &staker, 10_000)
        .await;
    assert!(result.is_err());
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let result = tr.claim_pool_rewards(&stake_pool_owner).await;
    assert!(result.is_err());

    // check balances
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 500_000);
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.balance, 500_000);

    // 1 hour later
    tr.sleep(3600).await.unwrap();

    // Crank should succeed
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim rewards again
    tr.claim_bond_rewards(&stake_pool_owner.pubkey(), &staker, 10_000)
        .await
        .unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 750_000);
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 1_000_000);
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.balance, 1_000_000);
}
