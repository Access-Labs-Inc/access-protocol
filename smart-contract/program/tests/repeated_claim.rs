use std::collections::HashMap;
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
async fn repeated_claim() {

    ///Please comment out the content of assert_authorized_seller otherwise this test will fail, src/utils.rs line 127
    /// There's an issue with the seller key created somewhere below

    // Setup the token + basic accounts
    let mut tr = TestRunner::new().await;

    // Create users
    let stake_pool_owner = tr.create_ata_account().await;
    let stake_pool2_owner = tr.create_ata_account().await;
    let staker = tr.create_ata_account().await;

    // Mint
    tr.mint(&staker.pubkey(), 10_200).await;

    // Create stake pool on day 1 12:00
    tr.create_stake_pool(&stake_pool_owner.pubkey()).await;

    // // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey()).await;

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey()).await;

    // Wait 1 hour
    tr.sleep(3600).await;

    // Create stake pool 2 on day 1 13:00
    tr.create_stake_pool(&stake_pool2_owner.pubkey()).await;

    // Activate stake pool 2
    tr.activate_stake_pool(&stake_pool2_owner.pubkey()).await;

    // Create stake account 2
    tr.create_stake_account(&stake_pool2_owner.pubkey(), &staker.pubkey()).await;

    // Stake to pool 1
    let token_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, token_amount).await;

    // wait until day 2 12:15
    tr.sleep(86400 - 2700).await;

    // Crank pool 1 (+ implicitly the whole system)
    tr.crank_pool(&stake_pool_owner.pubkey()).await;

    // Claim pool 1 rewards
    tr.claim_pool_rewards(&stake_pool_owner).await;

    // Claim staker rewards in pool 1
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker).await;

    // Unstake from pool 1
    tr.unstake(&stake_pool_owner.pubkey(), &staker, token_amount).await;

    // Crank pool 2
    tr.crank_pool(&stake_pool2_owner.pubkey()).await;

    // Stake 2
    tr.stake(&stake_pool2_owner.pubkey(), &staker, token_amount).await;

    // Claim stake pool rewards 2
    // tr.claim_pool_rewards(&stake_pool2_owner).await;

    // Claim rewards 2
    tr.claim_staker_rewards(&stake_pool2_owner.pubkey(), &staker).await;

    // Print results
    let stats = tr.staker_stats(staker.pubkey()).await;
    println!("[+] stats--->  {:?}", stats);
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await;
    println!("[+] pool_stats--->  {:?}", pool_stats);
    let pool_stats2 = tr.pool_stats(stake_pool2_owner.pubkey()).await;
    println!("[+] pool_stats2--->  {:?}", pool_stats2);

    // todo asserts
}


pub struct TestRunner {
    pub program_id: Pubkey,
    prg_test_ctx: ProgramTestContext,
    local_env: BanksClient,
    authority_ata: Pubkey,
    central_state: Pubkey,
    mint : Pubkey,
}

#[derive(Debug)]
pub struct StakerStats {
    balance: u64,
}

#[derive(Debug)]
pub struct PoolOwnerStats {
    balance: u64,
    total_pool_staked: u64,
}

impl TestRunner {
    pub async fn new() -> Self {
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


        Self {
            program_id,
            prg_test_ctx,
            local_env,
            authority_ata,
            central_state,
            mint,
        }
    }

    pub async fn create_ata_account(&mut self) -> Keypair {
        let owner = Keypair::new();
        let create_ata_stake_pool_owner_ix = create_associated_token_account(
            &self.prg_test_ctx.payer.pubkey(),
            &owner.pubkey(),
            &self.mint,
        );
        sign_send_instructions(
            &mut self.prg_test_ctx,
            vec![create_ata_stake_pool_owner_ix],
            vec![],
        )
            .await
            .unwrap();
        owner
    }

    pub async fn mint(&mut self, destination: &Pubkey, amount: u64) {
        let destination_ata = get_associated_token_address(&destination, &self.mint);
        let admin_mint_ix = admin_mint(
            self.program_id,
            admin_mint::Accounts {
                authority: &self.prg_test_ctx.payer.pubkey(),
                mint: &self.mint,
                access_token_destination: &destination_ata,
                central_state: &self.central_state,
                spl_token_program: &spl_token::ID,
            },
            admin_mint::Params {
                amount,
            },
        );
        sign_send_instructions(&mut self.prg_test_ctx, vec![admin_mint_ix], vec![])
            .await
            .unwrap();
    }

    pub async fn create_stake_pool(&mut self, stake_pool_owner: &Pubkey) {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let create_associated_instruction =
            create_associated_token_account(&self.prg_test_ctx.payer.pubkey(), &stake_pool_key, &self.mint);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);
        sign_send_instructions(
            &mut self.prg_test_ctx,
            vec![create_associated_instruction],
            vec![],
        )
            .await
            .unwrap();

        let create_stake_pool_ix = create_stake_pool(
            self.program_id,
            create_stake_pool::Accounts {
                stake_pool_account: &stake_pool_key,
                system_program: &system_program::ID,
                fee_payer: &self.prg_test_ctx.payer.pubkey(),
                vault: &pool_vault,
            },
            create_stake_pool::Params {
                owner: *stake_pool_owner,
                minimum_stake_amount: 1000,
            },
        );
        sign_send_instructions(&mut self.prg_test_ctx, vec![create_stake_pool_ix], vec![])
            .await
            .unwrap();
    }

    pub async fn activate_stake_pool(&mut self, stake_pool_owner: &Pubkey) {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let activate_stake_pool_ix = activate_stake_pool(
            self.program_id,
            activate_stake_pool::Accounts {
                authority: &self.prg_test_ctx.payer.pubkey(),
                stake_pool: &stake_pool_key,
                central_state: &self.central_state,
            },
            activate_stake_pool::Params {},
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![activate_stake_pool_ix], vec![])
            .await
            .unwrap();
    }

    pub fn get_stake_account_pda(&mut self, stake_pool_key: &Pubkey, staker_key: &Pubkey) -> (Pubkey, u8) {
        let (stake_acc_key, stake_nonce) = Pubkey::find_program_address(
            &[
                "stake_account".as_bytes(),
                &staker_key.to_bytes(),
                &stake_pool_key.to_bytes(),
            ],
            &self.program_id,
        );
        (stake_acc_key, stake_nonce)
    }

    pub fn get_pool_pda(&mut self, stake_pool_owner: &Pubkey) -> Pubkey {
        let (stake_pool_key, _) = Pubkey::find_program_address(
            &[
                "stake_pool".as_bytes(),
                &stake_pool_owner.to_bytes(),
            ],
            &self.program_id,
        );
        stake_pool_key
    }

    pub async fn create_stake_account(&mut self, stake_pool_owner_key: &Pubkey, staker_key: &Pubkey) {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner_key);
        let (stake_acc_key, stake_nonce) = self.get_stake_account_pda(&stake_pool_key, &staker_key);
        let create_stake_account_ix = create_stake_account(
            self.program_id,
            create_stake_account::Accounts {
                stake_account: &stake_acc_key,
                system_program: &system_program::ID,
                fee_payer: &self.prg_test_ctx.payer.pubkey(),
                stake_pool: &stake_pool_key,
            },
            create_stake_account::Params {
                nonce: stake_nonce,
                owner: *staker_key,
            },
        );
        sign_send_instructions(&mut self.prg_test_ctx, vec![create_stake_account_ix], vec![])
            .await
            .unwrap();
    }

    pub async fn sleep(&mut self, duration: u64) {
        self.prg_test_ctx.warp_to_timestamp(
            self.local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp + duration as i64
        ).await.unwrap();
    }

    pub async fn stake(&mut self, stake_pool_owner_key: &Pubkey, staker: &Keypair, token_amount: u64) {
        let staker_key = staker.pubkey();
        let stake_pool_key = self.get_pool_pda(stake_pool_owner_key);
        let (stake_acc_key, _) = self.get_stake_account_pda(&stake_pool_key, &staker_key);
        let staker_token_acc = get_associated_token_address(&staker_key, &self.mint);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);
        let stake_ix = stake(
            self.program_id,
            stake::Accounts {
                stake_account: &stake_acc_key,
                stake_pool: &stake_pool_key,
                owner: &staker_key,
                source_token: &staker_token_acc,
                spl_token_program: &spl_token::ID,
                vault: &pool_vault,
                central_state_account: &self.central_state,
                fee_account: &self.authority_ata,
            },
            stake::Params {
                amount: token_amount,
            },
        );
        sign_send_instructions(&mut self.prg_test_ctx, vec![stake_ix], vec![&staker])
            .await
            .unwrap();
    }

    pub async fn crank_pool(&mut self, stake_pool_owner_key: &Pubkey) {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner_key);
        let crank_ix = crank(
            self.program_id,
            crank::Accounts {
                stake_pool: &stake_pool_key,
                central_state: &self.central_state,
            },
            crank::Params {},
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![crank_ix], vec![])
            .await
            .unwrap();
    }

    pub async fn staker_stats(&mut self, staker_key: Pubkey) -> StakerStats {
        let staker_token_acc = get_associated_token_address(&staker_key, &self.mint);
        let balance = self.local_env.get_packed_account_data::<spl_token::state::Account>(staker_token_acc).await.unwrap().amount;
        StakerStats {
            balance
        }
    }

    pub async fn pool_stats(&mut self, stake_pool_owner: Pubkey) -> PoolOwnerStats {
        let stake_pool_owner_token_acc = get_associated_token_address(&stake_pool_owner, &self.mint);
        let balance = self.local_env.get_packed_account_data::<spl_token::state::Account>(stake_pool_owner_token_acc).await.unwrap().amount;

        let stake_pool_key = self.get_pool_pda(&stake_pool_owner);
        let stake_pool_associated_token_account = get_associated_token_address(&stake_pool_key, &self.mint);
        let total_pool_staked = self.local_env.get_packed_account_data::<spl_token::state::Account>(stake_pool_associated_token_account).await.unwrap().amount;

        PoolOwnerStats {
            balance,
            total_pool_staked,
        }
    }

    pub async fn claim_pool_rewards(&mut self, stake_pool_owner: &Keypair) {
        let stake_pool_key = self.get_pool_pda(&stake_pool_owner.pubkey());
        let stake_pool_owner_token_acc = get_associated_token_address(&stake_pool_owner.pubkey(), &self.mint);
        let claim_stake_pool_ix = claim_pool_rewards(
            self.program_id,
            claim_pool_rewards::Accounts {
                stake_pool: &stake_pool_key,
                owner: &stake_pool_owner.pubkey(),
                rewards_destination: &stake_pool_owner_token_acc,
                central_state: &self.central_state,
                mint: &self.mint,
                spl_token_program: &spl_token::ID,
            },
            claim_pool_rewards::Params {},
            true,
        );

        sign_send_instructions(
            &mut self.prg_test_ctx,
            vec![claim_stake_pool_ix],
            vec![&stake_pool_owner],
        )
            .await
            .unwrap();
    }

    pub async fn claim_staker_rewards(&mut self, stake_pool_owner: &Pubkey, staker: &Keypair) {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let (stake_acc_key, _) = self.get_stake_account_pda(&stake_pool_key, &staker.pubkey());
        let staker_token_acc = get_associated_token_address(&staker.pubkey(), &self.mint);

        let claim_ix = claim_rewards(
            self.program_id,
            claim_rewards::Accounts {
                stake_pool: &stake_pool_key,
                stake_account: &stake_acc_key,
                owner: &staker.pubkey(),
                rewards_destination: &staker_token_acc,
                central_state: &self.central_state,
                mint: &self.mint,
                spl_token_program: &spl_token::ID,
            },
            claim_rewards::Params {
                allow_zero_rewards: true,
            },
            true,
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![claim_ix], vec![&staker])
            .await
            .unwrap();
    }

    pub async fn unstake(&mut self, stake_pool_owner: &Pubkey, staker: &Keypair, token_amount: u64) {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let (stake_acc_key, _) = self.get_stake_account_pda(&stake_pool_key, &staker.pubkey());
        let staker_token_acc = get_associated_token_address(&staker.pubkey(), &self.mint);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);

        // Request Unstake
        let unstake_ix = unstake(
            self.program_id,
            unstake::Accounts {
                stake_account: &stake_acc_key,
                stake_pool: &stake_pool_key,
                owner: &staker.pubkey(),
                central_state_account: &self.central_state,
            },
            unstake::Params {
                amount: token_amount,
            },
        );
        sign_send_instructions(&mut self.prg_test_ctx, vec![unstake_ix], vec![&staker])
            .await
            .unwrap();

        // Execute Unstake
        let execute_unstake_ix = execute_unstake(
            self.program_id,
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
        sign_send_instructions(&mut self.prg_test_ctx, vec![execute_unstake_ix], vec![&staker])
            .await
            .unwrap();
    }
}