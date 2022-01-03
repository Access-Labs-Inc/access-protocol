use solana_program::{pubkey, pubkey::Pubkey, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::signer::{keypair::Keypair, Signer};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
pub mod common;
use crate::common::utils::{mint_bootstrap, sign_send_instructions};
use access_protocol::{
    entrypoint::process_instruction,
    instruction::{
        change_inflation, change_pool_minimum, claim_bond, claim_bond_rewards, claim_pool_rewards,
        claim_rewards, close_stake_account, close_stake_pool, crank, create_bond,
        create_central_state, create_stake_account, create_stake_pool, sign_bond, stake,
        unlock_bond_tokens, unstake,
    },
    state::BondAccount,
};

#[tokio::test]
async fn test_staking() {
    // Create program and test environment
    let program_id = pubkey!("hxrotrKwueSFofXvCmCpYyKMjn1BhmwKtPxA1nLcv8m");

    let mut program_test = ProgramTest::new(
        "access_protocol",
        program_id,
        processor!(process_instruction),
    );

    //
    // Derive central vault
    //
    let (central_state, _nonce) =
        Pubkey::find_program_address(&[&program_id.to_bytes()], &program_id);

    //
    // Create mint
    //
    let (mint, _) = mint_bootstrap(None, 6, &mut program_test, &central_state);

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

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

    let create_associated_instruction =
        create_associated_token_account(&prg_test_ctx.payer.pubkey(), &stake_pool_key, &mint);
    let pool_vault = get_associated_token_address(&stake_pool_key, &mint);
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![create_associated_instruction],
        vec![],
    )
    .await
    .unwrap();

    let create_stake_pool_ix = create_stake_pool(
        program_id,
        create_stake_pool::Accounts {
            stake_pool_account: &stake_pool_key,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            vault: &pool_vault,
        },
        create_stake_pool::Params {
            nonce: stake_pool_nonce,
            name: "stake_pool".to_string(),
            owner: stake_pool_owner.pubkey(),
            destination: stake_pool_owner_token_acc,
            minimum_stake_amount: 10_000_000,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_pool_ix], vec![])
        .await
        .unwrap();

    //
    // Create bond
    //

    let bond_amount = 50_000 * 1_000_000;
    let (bond_key, _bond_nonce) =
        BondAccount::create_key(&staker.pubkey(), bond_amount, &program_id);

    let create_bond_ix = create_bond(
        program_id,
        create_bond::Accounts {
            stake_pool: &stake_pool_key,
            seller: &prg_test_ctx.payer.pubkey(),
            bond_account: &bond_key,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
        },
        create_bond::Params {
            buyer: staker.pubkey(),
            total_amount_sold: bond_amount,
            seller_token_account: Pubkey::default(),
            total_quote_amount: 0,
            quote_mint: Pubkey::default(),
            unlock_period: 1,
            unlock_amount: bond_amount,
            unlock_start_date: 0,
            last_unlock_time: 1,
            stake_pool: stake_pool_key,
            seller_index: 0,
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![create_bond_ix], vec![])
        .await
        .unwrap();

    //
    // Claim bond
    //

    let claim_bond_ix = claim_bond(
        program_id,
        claim_bond::Accounts {
            bond_account: &bond_key,
            buyer: &staker.pubkey(),
            quote_token_source: &staker_token_acc,
            quote_token_destination: &stake_pool_owner_token_acc,
            spl_token_program: &spl_token::ID,
        },
        claim_bond::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_bond_ix], vec![&staker])
        .await
        .unwrap();

    //
    // Unlock bond tokens
    //

    let unlock_ix = unlock_bond_tokens(
        program_id,
        unlock_bond_tokens::Accounts {
            bond_account: &bond_key,
            bond_owner: &staker.pubkey(),
            mint: &mint,
            access_token_destination: &staker_token_acc,
            central_state: &central_state,
            spl_token_program: &spl_token::ID,
        },
        unlock_bond_tokens::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![unlock_ix], vec![&staker])
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
            stake_pool: &stake_pool_key,
        },
        create_stake_account::Params {
            nonce: stake_nonce,
            owner: staker.pubkey(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_account_ix], vec![])
        .await
        .unwrap();

    //
    // Stake
    //
    let token_amount = 10_000 * 1_000_000;

    let stake_ix = stake(
        program_id,
        stake::Accounts {
            stake_account: &stake_acc_key,
            stake_pool: &stake_pool_key,
            owner: &staker.pubkey(),
            source_token: &staker_token_acc,
            spl_token_program: &spl_token::ID,
            vault: &pool_vault,
        },
        stake::Params {
            amount: token_amount,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![stake_ix], vec![&staker])
        .await
        .unwrap();

    //
    // Crank
    //

    let crank_ix = crank(
        program_id,
        crank::Accounts {
            stake_pool: &stake_pool_key,
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
            spl_token_program: &spl_token::ID,
        },
        claim_pool_rewards::Params {},
    );

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![claim_stake_pool_ix],
        vec![&stake_pool_owner],
    )
    .await
    .unwrap();

    //
    // Claim bond rewards
    //

    let claim_bond_rewards_ix = claim_bond_rewards(
        program_id,
        claim_bond_rewards::Accounts {
            stake_pool: &stake_pool_key,
            bond_account: &bond_key,
            bond_owner: &staker.pubkey(),
            rewards_destination: &staker_token_acc,
            central_state: &central_state,
            mint: &mint,
            spl_token_program: &spl_token::ID,
        },
        claim_bond_rewards::Params {},
    );

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![claim_bond_rewards_ix],
        vec![&staker],
    )
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
            spl_token_program: &spl_token::ID,
        },
        claim_rewards::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_ix], vec![&staker])
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
    // Change minimum stake pool
    //

    let change_min_ix = change_pool_minimum(
        program_id,
        change_pool_minimum::Accounts {
            stake_pool: &stake_pool_key,
            stake_pool_owner: &stake_pool_owner.pubkey(),
        },
        change_pool_minimum::Params {
            new_minimum: 10_000_000 / 2,
        },
    );

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![change_min_ix],
        vec![&stake_pool_owner],
    )
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
            vault: &pool_vault,
        },
        unstake::Params {
            amount: token_amount,
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![unstake_ix], vec![&staker])
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

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![close_stake_account_ix],
        vec![&staker],
    )
    .await
    .unwrap();

    //
    // Close stake pool
    //

    let close_stake_pool_ix = close_stake_pool(
        program_id,
        close_stake_pool::Accounts {
            stake_pool_account: &stake_pool_key,
            owner: &stake_pool_owner.pubkey(),
        },
        close_stake_pool::Params {
            nonce: stake_pool_nonce,
            destination: stake_pool_owner_token_acc,
        },
    );

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![close_stake_pool_ix],
        vec![&stake_pool_owner],
    )
    .await
    .unwrap();
}
