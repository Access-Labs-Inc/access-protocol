use solana_program::{pubkey, pubkey::Pubkey, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::signer::{keypair::Keypair, Signer};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
pub mod common;
use crate::common::utils::{mint_bootstrap, sign_send_instructions};
use access_protocol::{
    entrypoint::process_instruction,
    instruction::{
        activate_stake_pool, admin_freeze, admin_mint, change_central_state_authority,
        change_inflation, change_pool_minimum, change_pool_multiplier, claim_bond,
        claim_bond_rewards, claim_pool_rewards, claim_rewards, close_stake_account,
        close_stake_pool, crank, create_bond, create_central_state, create_stake_account,
        create_stake_pool, execute_unstake, stake, unlock_bond_tokens, unstake,
    },
    state::{BondAccount, StakePool, FEES},
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
    // Create authority ACCESS token account
    //
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &prg_test_ctx.payer.pubkey(),
        &mint,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let authority_ata = get_associated_token_address(&&prg_test_ctx.payer.pubkey(), &mint);

    //
    // Create owner and user A
    //

    let stake_pool_owner = Keypair::new();
    let staker_a = Keypair::new();

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
        create_associated_token_account(&prg_test_ctx.payer.pubkey(), &staker_a.pubkey(), &mint);
    sign_send_instructions(&mut prg_test_ctx, vec![create_ata_staker_ix], vec![])
        .await
        .unwrap();

    let staker_token_acc_a = get_associated_token_address(&staker_a.pubkey(), &mint);
    let stake_pool_owner_token_acc = get_associated_token_address(&staker_a.pubkey(), &mint);

    //
    // Create user B
    //

    let stake_pool_owner = Keypair::new();
    let staker_b = Keypair::new();

    let create_ata_staker_ix =
        create_associated_token_account(&prg_test_ctx.payer.pubkey(), &staker_b.pubkey(), &mint);
    sign_send_instructions(&mut prg_test_ctx, vec![create_ata_staker_ix], vec![])
        .await
        .unwrap();

    let staker_token_acc_b = get_associated_token_address(&staker_b.pubkey(), &mint);

    //
    // Admin mint A
    //

    let admin_mint_ix = admin_mint(
        program_id,
        admin_mint::Accounts {
            authority: &prg_test_ctx.payer.pubkey(),
            mint: &mint,
            access_token_destination: &staker_token_acc_a,
            central_state: &central_state,
            spl_token_program: &spl_token::ID,
        },
        admin_mint::Params {
            amount: 10_000 * 1_000_000,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![admin_mint_ix], vec![])
        .await
        .unwrap();

    //
    // Admin mint B
    //

    let admin_mint_ix = admin_mint(
        program_id,
        admin_mint::Accounts {
            authority: &prg_test_ctx.payer.pubkey(),
            mint: &mint,
            access_token_destination: &staker_token_acc_b,
            central_state: &central_state,
            spl_token_program: &spl_token::ID,
        },
        admin_mint::Params {
            amount: 10_000 * 1_000_000,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![admin_mint_ix], vec![])
        .await
        .unwrap();

    //
    // Create stake pool
    //

    let (stake_pool_key, _) = Pubkey::find_program_address(
        &[
            "stake_pool".as_bytes(),
            &stake_pool_owner.pubkey().to_bytes(),
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
            owner: stake_pool_owner.pubkey(),
            minimum_stake_amount: 0,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_pool_ix], vec![])
        .await
        .unwrap();

    //
    // Activate stake pool
    //

    let activate_stake_pool_ix = activate_stake_pool(
        program_id,
        activate_stake_pool::Accounts {
            authority: &prg_test_ctx.payer.pubkey(),
            stake_pool: &stake_pool_key,
            central_state: &central_state,
        },
        activate_stake_pool::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![activate_stake_pool_ix], vec![])
        .await
        .unwrap();

    //
    // Create stake account A
    //

    let (stake_acc_key_a, stake_nonce_a) = Pubkey::find_program_address(
        &[
            "stake_account".as_bytes(),
            &staker_a.pubkey().to_bytes(),
            &stake_pool_key.to_bytes(),
        ],
        &program_id,
    );
    let create_stake_account_ix = create_stake_account(
        program_id,
        create_stake_account::Accounts {
            stake_account: &stake_acc_key_a,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            stake_pool: &stake_pool_key,
        },
        create_stake_account::Params {
            nonce: stake_nonce_a,
            owner: staker_a.pubkey(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_account_ix], vec![])
        .await
        .unwrap();

    //
    // Create stake account B
    //

    let (stake_acc_key_b, stake_nonce_b) = Pubkey::find_program_address(
        &[
            "stake_account".as_bytes(),
            &staker_b.pubkey().to_bytes(),
            &stake_pool_key.to_bytes(),
        ],
        &program_id,
    );
    let create_stake_account_ix = create_stake_account(
        program_id,
        create_stake_account::Accounts {
            stake_account: &stake_acc_key_b,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            stake_pool: &stake_pool_key,
        },
        create_stake_account::Params {
            nonce: stake_nonce_b,
            owner: staker_b.pubkey(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_account_ix], vec![])
        .await
        .unwrap();

    //
    // Stake A
    //
    let token_amount = 12000;

    let stake_ix = stake(
        program_id,
        stake::Accounts {
            stake_account: &stake_acc_key_a,
            stake_pool: &stake_pool_key,
            owner: &staker_a.pubkey(),
            source_token: &staker_token_acc_a,
            spl_token_program: &spl_token::ID,
            vault: &pool_vault,
            central_state_account: &central_state,
            fee_account: &authority_ata,
        },
        stake::Params {
            amount: token_amount,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![stake_ix], vec![&staker_a])
        .await
        .unwrap();

    //
    // Stake B
    //
    let token_amount = 12000;

    let stake_ix = stake(
        program_id,
        stake::Accounts {
            stake_account: &stake_acc_key_b,
            stake_pool: &stake_pool_key,
            owner: &staker_b.pubkey(),
            source_token: &staker_token_acc_b,
            spl_token_program: &spl_token::ID,
            vault: &pool_vault,
            central_state_account: &central_state,
            fee_account: &authority_ata,
        },
        stake::Params {
            amount: token_amount,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![stake_ix], vec![&staker_b])
        .await
        .unwrap();

    // Advance in time by a few seconds
    let mut current_slot = 5_000;
    prg_test_ctx.warp_to_slot(current_slot).unwrap();
    for _ in 0..10 {
        // Repeat to advance the timestamp by more than a full year.
        // Repetition is needed, warp_slot arg does not influence the runtime timestamp.
        current_slot += 5_000;
        prg_test_ctx.warp_to_slot(current_slot).unwrap();
    }

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

    // Advance in time by a few seconds
    current_slot += 5_000;
    prg_test_ctx.warp_to_slot(current_slot).unwrap();
    for _ in 0..10 {
        // Repeat to advance the timestamp by more than a full year.
        // Repetition is needed, warp_slot arg does not influence the runtime timestamp.
        current_slot += 5_000;
        prg_test_ctx.warp_to_slot(current_slot).unwrap();
    }

    let acc = prg_test_ctx
        .banks_client
        .get_account(stake_pool_key)
        .await
        .unwrap();
    let stakd = StakePool::from_buffer(&acc.unwrap().data);
    println!("{:?}", stakd);

    //
    // Claim rewards A
    //

    let claim_ix = claim_rewards(
        program_id,
        claim_rewards::Accounts {
            stake_pool: &stake_pool_key,
            stake_account: &stake_acc_key_a,
            owner: &staker_a.pubkey(),
            rewards_destination: &staker_token_acc_a,
            central_state: &central_state,
            mint: &mint,
            spl_token_program: &spl_token::ID,
        },
        claim_rewards::Params { nb_days: 4 },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_ix], vec![&staker_a])
        .await
        .unwrap();

    //
    // Claim rewards B
    //

    let claim_ix = claim_rewards(
        program_id,
        claim_rewards::Accounts {
            stake_pool: &stake_pool_key,
            stake_account: &stake_acc_key_b,
            owner: &staker_b.pubkey(),
            rewards_destination: &staker_token_acc_b,
            central_state: &central_state,
            mint: &mint,
            spl_token_program: &spl_token::ID,
        },
        claim_rewards::Params { nb_days: 1 },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_ix], vec![&staker_b])
        .await
        .unwrap();
}
