use std::os::macos::raw::stat;
use solana_program::{pubkey::Pubkey, system_program};
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
use access_protocol::instruction::create_bond;

#[tokio::test]
async fn common_stake_limit() {
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

    // try staking to pool 1 under the stake limit
    let result = tr.stake(&stake_pool_owner.pubkey(), &staker, 999).await;
    assert!(result.is_err());

    // try staking to pool 1 on the stake limit
    tr.stake(&stake_pool_owner.pubkey(), &staker, 1000).await.unwrap();

    // unstake
    tr.unstake(&stake_pool_owner.pubkey(), &staker, 1000).await.unwrap();

    // Create bond account
    tr.create_bond(&stake_pool_owner.pubkey(), &staker.pubkey(), 10_000).await.unwrap();

    // Claim bond
    tr.claim_bond(&stake_pool_owner.pubkey(), &staker, 10_000).await.unwrap();

    // staking under the stake limit should work
    tr.stake(&stake_pool_owner.pubkey(), &staker, 1).await.unwrap();
}