use solana_program::{pubkey, pubkey::Pubkey, system_program, sysvar};
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
        create_stake_pool, edit_metadata, execute_unstake, stake, unlock_bond_tokens, unstake,
    },
    state::{BondAccount},
};
use mpl_token_metadata::{pda::find_metadata_account, instruction::create_metadata_accounts_v3};

#[tokio::test]
async fn test_staking() {
    // Create program and test environment
    let program_id = pubkey!("hxrotrKwueSFofXvCmCpYyKMjn1BhmwKtPxA1nLcv8m");

    let mut program_test = ProgramTest::new(
        "access_protocol",
        program_id,
        processor!(process_instruction),
    );

    program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);

    //
    // Derive central vault
    //
    let (central_state, _nonce) =
        Pubkey::find_program_address(&[&program_id.to_bytes()], &program_id);

    // let authority = Keypair::new();

    //
    // Create mint
    //
    let (mint, _) = mint_bootstrap(None, 6, &mut program_test, &central_state);

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    ////
    // Metadata account
    ////
    let (metadata_key, _) = find_metadata_account(&mint);

    //
    // Create central state
    //
    let daily_inflation: u64 = 1_000_000 * 500_000;
    let create_central_state_ix = create_central_state(
        program_id,
        create_central_state::Accounts {
            central_state: &central_state,
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

    ////
    // Metadata creation
    ////
    /// TODO: (ladi) To bring back the metadata test we need to initialize the SPL token
    /// with it's own authority and than change the authority to central_state.
    /// Because the Mint is created via add_account() I'm not sure if we can actually
    /// change the authority with set-new-authority equivalent from CLI.
    /// 
    // let create_metadata_ix = create_metadata_accounts_v3(
    //     mpl_token_metadata::ID,
    //     metadata_key,
    //     mint,
    //     authority.pubkey(),
    //     prg_test_ctx.payer.pubkey(),
    //     authority.pubkey(),
    //     "Access Protocol".to_string(),
    //     "ACS".to_string(),
    //     "URI".to_string(),
    //     None,
    //     0,
    //     true,
    //     true,
    //     None,
    //     None,
    //     None,
    // );

    // sign_send_instructions(&mut prg_test_ctx, vec![create_metadata_ix], vec![&authority])
    //     .await
    //     .unwrap();

    //
    // Edit metadata
    //
    // let ix = edit_metadata(
    //     program_id,
    //     edit_metadata::Accounts {
    //         central_state: &central_state,
    //         authority: &authority.pubkey(),
    //         metadata: &metadata_key,
    //         metadata_program: &mpl_token_metadata::ID,
    //     },
    //     edit_metadata::Params {
    //         name: "New name".to_string(),
    //         symbol: "New symbol".to_string(),
    //         uri: "New uri".to_string(),
    //     },
    // );
    // sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&authority])
    //     .await
    //     .unwrap();

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
            minimum_stake_amount: 10_000_000,
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
    // Change pool multiplier
    //
    let ix = change_pool_multiplier(
        program_id,
        change_pool_multiplier::Accounts {
            stake_pool: &stake_pool_key,
            stake_pool_owner: &stake_pool_owner.pubkey(),
        },
        change_pool_multiplier::Params { new_multiplier: 2 },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&stake_pool_owner])
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
            seller_token_account: stake_pool_owner_token_acc,
            total_quote_amount: 0,
            quote_mint: Pubkey::default(),
            unlock_period: 1,
            unlock_amount: bond_amount,
            unlock_start_date: 0,
            seller_index: 0,
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![create_bond_ix], vec![])
        .await
        .unwrap();

    //
    // Reject creating bond with unlock period eq to 0
    // as unlocking would not work (division by 0)
    //
    let create_bond_with_unlock_period_zero_ix = create_bond(
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
            seller_token_account: stake_pool_owner_token_acc,
            total_quote_amount: 0,
            quote_mint: Pubkey::default(),
            unlock_period: 0, // <- not allowed
            unlock_amount: bond_amount,
            unlock_start_date: 0,
            seller_index: 0,
        },
    );

    assert!(sign_send_instructions(
        &mut prg_test_ctx,
        vec![create_bond_with_unlock_period_zero_ix],
        vec![]
    )
    .await
    .is_err());

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
            stake_pool: &stake_pool_key,
            access_mint: &mint,
            pool_vault: &pool_vault,
            central_state: &central_state,
        },
        claim_bond::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_bond_ix], vec![&staker])
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
    let token_amount = 10_000_000;

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
        false,
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_bond_rewards_ix], vec![])
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
            allow_zero_rewards: false,
        },
        true,
    );

    sign_send_instructions(&mut prg_test_ctx, vec![claim_ix], vec![&staker])
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
            stake_pool: &stake_pool_key,
            pool_vault: &pool_vault,
        },
        unlock_bond_tokens::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![unlock_ix], vec![&staker])
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
            amount: token_amount,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![unstake_ix], vec![&staker])
        .await
        .unwrap();

    // Advance in time by a few seconds
    current_slot += 5_000;
    prg_test_ctx.warp_to_slot(current_slot).unwrap();

    //
    // Execute Unstake
    //

    let execute_unstake_ix = execute_unstake(
        program_id,
        execute_unstake::Accounts {
            stake_account: &stake_acc_key,
            stake_pool: &stake_pool_key,
            owner: &staker.pubkey(),
            destination_token: &staker_token_acc,
            spl_token_program: &spl_token::ID,
            vault: &pool_vault,
        },
        execute_unstake::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![execute_unstake_ix], vec![&staker])
        .await
        .unwrap();

    //
    // Freeze account
    //

    let freeze_stake_acc_ix = admin_freeze(
        program_id,
        admin_freeze::Accounts {
            central_state: &central_state,
            account_to_freeze: &stake_pool_key,
            authority: &prg_test_ctx.payer.pubkey(),
        },
        admin_freeze::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![freeze_stake_acc_ix], vec![])
        .await
        .unwrap();

    // Advance in time by a few seconds
    current_slot += 5000;
    prg_test_ctx.warp_to_slot(current_slot).unwrap();

    //
    // Unfreeze account
    //

    let freeze_stake_acc_ix = admin_freeze(
        program_id,
        admin_freeze::Accounts {
            central_state: &central_state,
            account_to_freeze: &stake_pool_key,
            authority: &prg_test_ctx.payer.pubkey(),
        },
        admin_freeze::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![freeze_stake_acc_ix], vec![])
        .await
        .unwrap();

    //
    // Try to freeze the central state (expected to fail)
    //

    let freeze_stake_acc_ix = admin_freeze(
        program_id,
        admin_freeze::Accounts {
            central_state: &central_state,
            account_to_freeze: &central_state,
            authority: &prg_test_ctx.payer.pubkey(),
        },
        admin_freeze::Params {},
    );

    assert!(
        sign_send_instructions(&mut prg_test_ctx, vec![freeze_stake_acc_ix], vec![])
            .await
            .is_err()
    );

    //
    // Close stake account
    //

    let close_stake_account_ix = close_stake_account(
        program_id,
        close_stake_account::Accounts {
            stake_account: &stake_acc_key,
            owner: &staker.pubkey(),
        },
        close_stake_account::Params {},
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
            pool_vault: &pool_vault,
            stake_pool_account: &stake_pool_key,
            owner: &stake_pool_owner.pubkey(),
        },
        close_stake_pool::Params {},
    );

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![close_stake_pool_ix],
        vec![&stake_pool_owner],
    )
    .await
    .unwrap();

    //
    // Change central state auth
    //
    let ix = change_central_state_authority(
        program_id,
        change_central_state_authority::Accounts {
            central_state: &central_state,
            authority: &prg_test_ctx.payer.pubkey(),
        },
        change_central_state_authority::Params {
            new_authority: Keypair::new().pubkey(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
}
