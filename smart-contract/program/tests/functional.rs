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
}
