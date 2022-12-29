use solana_program::{msg, pubkey, pubkey::Pubkey, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_test_framework::*;
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::sysvar::{clock};
use spl_associated_token_account::{instruction::create_associated_token_account, get_associated_token_address};
pub mod common;
use crate::common::utils::{mint_bootstrap, sign_send_instructions};
use access_protocol::{
    entrypoint::process_instruction,
    instruction::{
        activate_stake_pool, admin_freeze, admin_mint, change_central_state_authority,
        change_inflation, change_pool_minimum, change_pool_multiplier, claim_bond,
        claim_bond_rewards, claim_pool_rewards, claim_rewards, close_stake_account,
        close_stake_pool, crank, create_bond, create_central_state, create_stake_account,
        create_stake_pool, edit_metadata, execute_unstake, stake, unlock_bond_tokens, unstake,
    },
    state::{BondAccount, FEES},
};
use mpl_token_metadata::pda::find_metadata_account;

#[tokio::test]
async fn halborn() {

    ///Please comment out the content of assert_authorized_seller otherwise this test will fail, src/utils.rs line 127
    /// There's an issue with the seller key created somewhere below

    // Create program and test environment
    let program_id = access_protocol::ID;

    let mut program_test = ProgramTest::default();

    program_test.prefer_bpf(true);

    // todo make this relative
    program_test.add_program(
        "/Users/matusvla/go/src/github.com/Access-Labs-Inc/access-protocol/smart-contract/program/target/deploy/access_protocol",
        access_protocol::ID,
        processor!(process_instruction),
    );
    println!("added access_protocol::ID {:?}", access_protocol::ID);

    // todo make this relative
    program_test.add_program("/Users/matusvla/go/src/github.com/Access-Labs-Inc/accessprotocol.co/metaplex-program-library/token-metadata/target/deploy/mpl_token_metadata",   mpl_token_metadata::ID, None);
    println!("added mpl_token_metadata::ID {:?}", mpl_token_metadata::ID);

    //
    // Derive central vault
    //
    let (central_state, _nonce) =
        Pubkey::find_program_address(&[&program_id.to_bytes()], &program_id);

    //
    // Create mint
    //
    let (mint, _) = mint_bootstrap(Some("acsT7dFjiyevrBbvpsD7Vqcwj1QN96fbWKdq49wcdWZ"), 6, &mut program_test, &central_state);

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;
    let mut local_env = prg_test_ctx.banks_client.clone(); 
    

    ////
    // Metadata account
    ////
    let (metadata_key, _) = find_metadata_account(&mint);

    //
    // Create central state
    //
    let daily_inflation: u64 = 1_000_000;
    let create_central_state_ix = create_central_state(
        program_id,
        create_central_state::Accounts {
            central_state: &central_state,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            mint: &mint,
            metadata: &metadata_key,
            metadata_program: &mpl_token_metadata::ID,
            rent_sysvar: &solana_program::sysvar::rent::ID,
        },
        create_central_state::Params {
            daily_inflation,
            authority: prg_test_ctx.payer.pubkey(),
            name: "Access protocol token".to_string(),
            symbol: "ACCESS".to_string(),
            uri: "uri".to_string(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_central_state_ix], vec![])
        .await
        .unwrap();

    //
    // Edit metadata
    //
    let ix = edit_metadata(
        program_id,
        edit_metadata::Accounts {
            central_state: &central_state,
            authority: &prg_test_ctx.payer.pubkey(),
            metadata: &metadata_key,
            metadata_program: &mpl_token_metadata::ID,
        },
        edit_metadata::Params {
            name: "New name".to_string(),
            symbol: "New symbol".to_string(),
            uri: "New uri".to_string(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
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
    // Create users
    //

    let stake_pool_owner = Keypair::new();
    let stake_pool_owner2 = Keypair::new();
    let staker = Keypair::new();

    println!("stake_pool_owner {:?}", stake_pool_owner.pubkey());
    println!("stake_pool_owner2 {:?}", stake_pool_owner2.pubkey());
    println!("staker {:?}", staker.pubkey());

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

    let create_ata_stake_pool_owner_ix2 = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &stake_pool_owner2.pubkey(),
        &mint,
    );
    sign_send_instructions(
        &mut prg_test_ctx,
        vec![create_ata_stake_pool_owner_ix2],
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
    let stake_pool_owner_token_acc = get_associated_token_address(&stake_pool_owner.pubkey(), &mint);
    let stake_pool_owner2_token_acc = get_associated_token_address(&stake_pool_owner2.pubkey(), &mint);

    //
    // Admin mint
    //

    let admin_mint_ix = admin_mint(
        program_id,
        admin_mint::Accounts {
            authority: &prg_test_ctx.payer.pubkey(),
            mint: &mint,
            access_token_destination: &staker_token_acc,
            central_state: &central_state,
            spl_token_program: &spl_token::ID,
        },
        admin_mint::Params {
            amount: 10_200,
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

    println!("stake_pool_key: {}", stake_pool_key);

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
            minimum_stake_amount: 1000,
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
    println!("Stake account key: {}", stake_acc_key);
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
    // Create stake pool 2
    //

    prg_test_ctx.warp_to_timestamp(
        local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp + 2600 //+ 1 hour to 13:00 on day 1
    ).await.unwrap();

    let (stake_pool_key2, _) = Pubkey::find_program_address(
        &[
            "stake_pool".as_bytes(),
            &stake_pool_owner2.pubkey().to_bytes(),
        ],
        &program_id,
    );

    let create_associated_instruction =
        create_associated_token_account(&prg_test_ctx.payer.pubkey(), &stake_pool_key2, &mint);
    let pool_vault2 = get_associated_token_address(&stake_pool_key2, &mint);
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
            stake_pool_account: &stake_pool_key2,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            vault: &pool_vault2,
        },
        create_stake_pool::Params {
            owner: stake_pool_owner2.pubkey(),
            minimum_stake_amount: 1000,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_pool_ix], vec![])
        .await
        .unwrap();

    //
    // Activate stake pool 2
    //

    let activate_stake_pool_ix = activate_stake_pool(
        program_id,
        activate_stake_pool::Accounts {
            authority: &prg_test_ctx.payer.pubkey(),
            stake_pool: &stake_pool_key2,
            central_state: &central_state,
        },
        activate_stake_pool::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![activate_stake_pool_ix], vec![])
        .await
        .unwrap();

    //
    // Create stake account 2
    //

    let (stake_acc_key2, stake_nonce2) = Pubkey::find_program_address(
        &[
            "stake_account".as_bytes(),
            &staker.pubkey().to_bytes(),
            &stake_pool_key2.to_bytes(),
        ],
        &program_id,
    );
    let create_stake_account_ix = create_stake_account(
        program_id,
        create_stake_account::Accounts {
            stake_account: &stake_acc_key2,
            system_program: &system_program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            stake_pool: &stake_pool_key2,
        },
        create_stake_account::Params {
            nonce: stake_nonce2,
            owner: staker.pubkey(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![create_stake_account_ix], vec![])
        .await
        .unwrap();

    //
    // Stake
    //
    let token_amount = 10_000;

    let stake_ix = stake(
        program_id,
        stake::Accounts {
            stake_account: &stake_acc_key,
            stake_pool: &stake_pool_key,
            owner: &staker.pubkey(),
            source_token: &staker_token_acc,
            spl_token_program: &spl_token::ID,
            vault: &pool_vault,
            central_state_account: &central_state,
            fee_account: &authority_ata,
        },
        stake::Params {
            amount: token_amount,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![stake_ix], vec![&staker])
        .await
        .unwrap();

    prg_test_ctx.warp_to_timestamp(
        local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp + 83115 //change time to 12:15 on day 2 (I had to add an extra 300 seconds due to other ixs)
    ).await.unwrap();
    

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

    prg_test_ctx.warp_to_timestamp(
        local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp + 900 //change time to 12:30 on day 2
    ).await.unwrap();

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
        true,
    );

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![claim_stake_pool_ix],
        vec![&stake_pool_owner],
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
        claim_rewards::Params {
            allow_zero_rewards: true,
        },
        true,
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_ix], vec![&staker])
        .await
        .unwrap();

    //
    // Request Unstake
    //

    let unstake_ix = unstake(
        program_id,
        unstake::Accounts {
            stake_account: &stake_acc_key,
            stake_pool: &stake_pool_key,
            owner: &staker.pubkey(),
            central_state_account: &central_state,
        },
        unstake::Params {
            amount: 10_000 as u64,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![unstake_ix], vec![&staker])
        .await
        .unwrap();

    //
    // Crank 2
    //

    let crank_ix = crank(
        program_id,
        crank::Accounts {
            stake_pool: &stake_pool_key2,
            central_state: &central_state,
        },
        crank::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![crank_ix], vec![])
        .await
        .unwrap();

    //
    // Stake 2
    //
    let token_amount = 10_000;

    let stake_ix = stake(
        program_id,
        stake::Accounts {
            stake_account: &stake_acc_key2,
            stake_pool: &stake_pool_key2,
            owner: &staker.pubkey(),
            source_token: &staker_token_acc,
            spl_token_program: &spl_token::ID,
            vault: &pool_vault2,
            central_state_account: &central_state,
            fee_account: &authority_ata,
        },
        stake::Params {
            amount: token_amount,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![stake_ix], vec![&staker])
        .await
        .unwrap();

        prg_test_ctx.warp_to_timestamp(
            local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp + 2700 //change time to 13:15 on day 2
        ).await.unwrap();

    //
    // Claim stake pool rewards 2
    //

    prg_test_ctx.warp_to_timestamp(
        local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp + 900 //change time to 13:30 on day 2
    ).await.unwrap();

    let claim_stake_pool_ix = claim_pool_rewards(
        program_id,
        claim_pool_rewards::Accounts {
            stake_pool: &stake_pool_key2,
            owner: &stake_pool_owner2.pubkey(),
            rewards_destination: &stake_pool_owner2_token_acc,
            central_state: &central_state,
            mint: &mint,
            spl_token_program: &spl_token::ID,
        },
        claim_pool_rewards::Params {},
        true,
    );

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![claim_stake_pool_ix],
        vec![&stake_pool_owner2],
    )
    .await
    .unwrap();

    //
    // Claim rewards 2
    //

    let claim_ix = claim_rewards(
        program_id,
        claim_rewards::Accounts {
            stake_pool: &stake_pool_key2,
            stake_account: &stake_acc_key2,
            owner: &staker.pubkey(),
            rewards_destination: &staker_token_acc,
            central_state: &central_state,
            mint: &mint,
            spl_token_program: &spl_token::ID,
        },
        claim_rewards::Params {
            allow_zero_rewards: true,
        },
        true,
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_ix], vec![&staker])
        .await
        .unwrap();

    //
    // Request Unstake
    //

    let unstake_ix = unstake(
        program_id,
        unstake::Accounts {
            stake_account: &stake_acc_key2,
            stake_pool: &stake_pool_key2,
            owner: &staker.pubkey(),
            central_state_account: &central_state,
        },
        unstake::Params {
            amount: 9_000 as u64,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![unstake_ix], vec![&staker])
        .await
        .unwrap();

    println!("[+] staker_token_acc funds--->  {:?}", local_env.get_packed_account_data::<spl_token::state::Account>(staker_token_acc).await.unwrap().amount);
    println!("[+] stake_pool_owner_token_acc funds--->  {:?}", local_env.get_packed_account_data::<spl_token::state::Account>(stake_pool_owner_token_acc).await.unwrap().amount);
    println!("[+] stake_pool_owner2_token_acc funds--->  {:?}", local_env.get_packed_account_data::<spl_token::state::Account>(stake_pool_owner2_token_acc).await.unwrap().amount);
    println!("[+] pool_vault funds--->  {:?}", local_env.get_packed_account_data::<spl_token::state::Account>(pool_vault).await.unwrap().amount);
    println!("[+] pool_vault2 funds--->  {:?}", local_env.get_packed_account_data::<spl_token::state::Account>(pool_vault2).await.unwrap().amount);
    println!("[+] authority_ata funds--->  {:?}", local_env.get_packed_account_data::<spl_token::state::Account>(authority_ata).await.unwrap().amount);
    
    

}
