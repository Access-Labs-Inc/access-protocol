use solana_program::{ pubkey::Pubkey, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_test_framework::*;
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::sysvar::{clock};
use spl_associated_token_account::{instruction::create_associated_token_account, get_associated_token_address};
pub mod common;
use crate::common::utils::{mint_bootstrap, sign_send_instructions};
use crate::common::test_runner::{TestRunner};
use access_protocol::{
    entrypoint::process_instruction,
    instruction::{
        activate_stake_pool, admin_mint,
        claim_pool_rewards, claim_rewards,
        crank, create_central_state, create_stake_account,
        create_stake_pool, execute_unstake, stake, unstake,
    },
};
use mpl_token_metadata::pda::find_metadata_account;

#[tokio::test]
async fn repeated_claim() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new().await.unwrap();

    // Create users
    let stake_pool_owner = tr.create_ata_account().await.unwrap();
    let stake_pool2_owner = tr.create_ata_account().await.unwrap();
    let staker = tr.create_ata_account().await.unwrap();

    // Mint
    tr.mint(&staker.pubkey(), 10_200).await.unwrap();

    // Create stake pool on day 1 12:00
    tr.create_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey()).await.unwrap();

    // Wait 1 hour
    tr.sleep(3600).await.unwrap();

    // Create stake pool 2 on day 1 13:00
    tr.create_stake_pool(&stake_pool2_owner.pubkey()).await.unwrap();

    // Activate stake pool 2
    tr.activate_stake_pool(&stake_pool2_owner.pubkey()).await.unwrap();

    // Create stake account 2
    tr.create_stake_account(&stake_pool2_owner.pubkey(), &staker.pubkey()).await.unwrap();

    // Stake to pool 1
    let token_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, token_amount).await.unwrap();

    // wait until day 2 12:15
    tr.sleep(86400 - 2700).await.unwrap();

    // Crank pool 1 (+ implicitly the whole system)
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim pool 1 rewards
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();

    // Claim staker rewards in pool 1
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker).await.unwrap();

    // Unstake from pool 1
    tr.unstake(&stake_pool_owner.pubkey(), &staker, token_amount).await.unwrap();

    // Crank pool 2
    tr.crank_pool(&stake_pool2_owner.pubkey()).await.unwrap();

    // Stake 2
    tr.stake(&stake_pool2_owner.pubkey(), &staker, token_amount).await.unwrap();

    // Claim stake pool rewards 2
    assert!(tr.claim_pool_rewards(&stake_pool2_owner).await.is_err());

    // Claim rewards 2
    tr.claim_staker_rewards(&stake_pool2_owner.pubkey(), &staker).await.unwrap();

    // Print results
    let stats = tr.staker_stats(staker.pubkey()).await;
    println!("[+] stats--->  {:?}", stats);
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await;
    println!("[+] pool_stats--->  {:?}", pool_stats);
    let pool_stats2 = tr.pool_stats(stake_pool2_owner.pubkey()).await;
    println!("[+] pool_stats2--->  {:?}", pool_stats2);

    // todo asserts
}