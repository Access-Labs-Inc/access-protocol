use solana_program::{
    instruction::InstructionError, pubkey, pubkey::Pubkey, rent::Rent, system_program, sysvar,
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    signer::{keypair::Keypair, Signer},
    transaction::TransactionError,
    transport::TransportError,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::instruction::mint_to;
use std::time::{SystemTime, UNIX_EPOCH};
pub mod common;
use crate::common::utils::{mint_bootstrap, sign_send_instructions};
use the_block::{
    entrypoint::process_instruction,
    instruction::{
        change_inflation, claim_pool_rewards, claim_rewards, close_stake_account, close_stake_pool,
        crank, create_central_state, create_stake_account, create_stake_pool, stake, unstake,
    },
};

#[tokio::test]
async fn test_staking() {
    // Create program and test environment
    let program_id = pubkey!("hxrotrKwueSFofXvCmCpYyKMjn1BhmwKtPxA1nLcv8m");

    let mut program_test =
        ProgramTest::new("the_block", program_id, processor!(process_instruction));

    //
    // Create mint
    //
    let mint_authority = Keypair::new();
    let (mint, _) = mint_bootstrap(None, 6, &mut program_test, &mint_authority.pubkey());

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    //
    // Derive central vault
    //
    let (central_state, nonce) =
        Pubkey::find_program_address(&[&program_id.to_bytes()], &program_id);

    //
    // Create central vault
    //
    let create_associated_instruction =
        create_associated_token_account(&prg_test_ctx.payer.pubkey(), &central_state, &mint);
    let vault = get_associated_token_address(&central_state, &mint);
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![create_associated_instruction],
        vec![],
    )
    .await
    .unwrap();

    //
    // Create central state
    //
    let daily_inflation: u64 = 1_000_000 * 500_000;
    let create_central_state_ix = create_central_state(
        program_id,
        create_central_state::Accounts {
            state_account: &central_state,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            rent_sysvar_account: &sysvar::rent::ID,
            central_vault: &vault,
            mint: &mint,
        },
        create_central_state::Params {
            daily_inflation,
            authority: prg_test_ctx.payer.pubkey(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_central_state_ix], vec![])
        .await
        .unwrap();

    //
    // Create users
    //

    let stake_pool_owner = Keypair::new();
    let staker = Keypair::new();

    let create_ata_stake_pool_owner_ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &stake_pool_owner.pubkey(),
        &mint,
    );
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![create_ata_stake_pool_owner_ix],
        vec![],
    )
    .await
    .unwrap();

    let create_ata_staker_ix =
        create_associated_token_account(&prg_test_ctx.payer.pubkey(), &staker.pubkey(), &mint);
    sign_send_instructions(&mut prg_test_ctx, vec![create_ata_staker_ix], vec![])
        .await
        .unwrap();

    let staker_token_acc = get_associated_token_address(&staker.pubkey(), &mint);
    let stake_pool_owner_token_acc = get_associated_token_address(&staker.pubkey(), &mint);

    //
    // Create stake pool
    //

    let (stake_pool_key, stake_pool_nonce) = Pubkey::find_program_address(
        &[
            "stake_pool".as_bytes(),
            &stake_pool_owner.pubkey().to_bytes(),
            &stake_pool_owner_token_acc.to_bytes(),
        ],
        &program_id,
    );
    let create_stake_pool_ix = create_stake_pool(
        program_id,
        create_stake_pool::Accounts {
            stake_pool_account: &stake_pool_key,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            rent_sysvar_account: &sysvar::rent::ID,
            vault: &vault,
        },
        create_stake_pool::Params {
            nonce: stake_pool_nonce,
            name: "stake pool".to_string(),
            owner: stake_pool_owner.pubkey(),
            destination: stake_pool_owner_token_acc,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_pool_ix], vec![])
        .await
        .unwrap();

    //
    // Create stake account
    //

    let (stake_acc_key, stake_nonce) = Pubkey::find_program_address(
        &[
            "stake_account".as_bytes(),
            &staker.pubkey().to_bytes(),
            &stake_pool_key.to_bytes(),
        ],
        &program_id,
    );
    let create_stake_account_ix = create_stake_account(
        program_id,
        create_stake_account::Accounts {
            stake_account: &stake_acc_key,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            rent_sysvar_account: &sysvar::rent::ID,
        },
        create_stake_account::Params {
            nonce: stake_nonce,
            owner: staker.pubkey(),
            stake_pool: stake_acc_key,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_account_ix], vec![])
        .await
        .unwrap();

    //
    // Stake
    //
    let token_amount = 10_000 * 1_000_000;
    let mint_ix = mint_to(
        &spl_token::ID,
        &mint,
        &staker_token_acc,
        &staker.pubkey(),
        &[],
        token_amount,
    )
    .unwrap();

    sign_send_instructions(&mut prg_test_ctx, vec![mint_ix], vec![&mint_authority])
        .await
        .unwrap();

    let stake_ix = stake(
        program_id,
        stake::Accounts {
            stake_account: &stake_acc_key,
            stake_pool: &stake_pool_key,
            owner: &staker.pubkey(),
            source_token: &staker_token_acc,
            spl_token_program: &spl_token::ID,
            vault: &vault,
        },
        stake::Params {
            amount: token_amount,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![stake_ix], vec![])
        .await
        .unwrap();

    //
    // Crank
    //

    let crank_ix = crank(
        program_id,
        crank::Accounts {
            stake_pool: &stake_acc_key,
            central_state: &central_state,
        },
        crank::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![crank_ix], vec![])
        .await
        .unwrap();

    //
    // Claim stake pool rewards
    //

    let claim_stake_pool_ix = claim_pool_rewards(
        program_id,
        claim_pool_rewards::Accounts {
            stake_pool: &stake_pool_key,
            owner: &stake_pool_owner.pubkey(),
            rewards_destination: &stake_pool_owner_token_acc,
            central_state: &central_state,
            mint: &mint,
            central_vault: &vault,
            spl_token_program: &spl_token::ID,
        },
        claim_pool_rewards::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_stake_pool_ix], vec![])
        .await
        .unwrap();

    //
    // Claim rewards
    //

    let claim_ix = claim_rewards(
        program_id,
        claim_rewards::Accounts {
            stake_pool: &stake_pool_key,
            stake_account: &stake_acc_key,
            owner: &staker.pubkey(),
            rewards_destination: &staker_token_acc,
            central_state: &central_state,
            mint: &mint,
            central_vault: &vault,
            spl_token_program: &spl_token::ID,
        },
        claim_rewards::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_ix], vec![])
        .await
        .unwrap();

    //
    // Change inflation
    //
    let new_inflation = 2 * daily_inflation;
    let change_inflation_ix = change_inflation(
        program_id,
        change_inflation::Accounts {
            central_state: &central_state,
            authority: &prg_test_ctx.payer.pubkey(),
        },
        change_inflation::Params {
            daily_inflation: new_inflation,
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![change_inflation_ix], vec![])
        .await
        .unwrap();

    //
    // Unstake
    //

    let unstake_ix = unstake(
        program_id,
        unstake::Accounts {
            stake_account: &stake_acc_key,
            stake_pool: &stake_pool_key,
            owner: &staker.pubkey(),
            destination_token: &staker_token_acc,
            spl_token_program: &spl_token::ID,
            vault: &vault,
        },
        unstake::Params {
            amount: token_amount,
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![unstake_ix], vec![])
        .await
        .unwrap();

    //
    // Close stake account
    //

    let close_stake_account_ix = close_stake_account(
        program_id,
        close_stake_account::Accounts {
            stake_account: &stake_acc_key,
            owner: &staker.pubkey(),
        },
        close_stake_account::Params {
            nonce: stake_nonce,
            owner: staker.pubkey(),
            stake_pool: stake_pool_key,
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![close_stake_account_ix], vec![])
        .await
        .unwrap();

    //
    // Close stake pool
    //

    let close_stake_pool_ix = close_stake_pool(
        program_id,
        close_stake_pool::Accounts {
            stake_pool_account: &stake_pool_key,
            system_program: &system_program::ID,
            owner: &stake_pool_owner.pubkey(),
        },
        close_stake_pool::Params {
            nonce: stake_pool_nonce,
            destination: stake_pool_owner_token_acc,
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![close_stake_pool_ix], vec![])
        .await
        .unwrap();
}
