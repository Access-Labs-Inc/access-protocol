use solana_program::{ pubkey::Pubkey, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_test_framework::*;
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::sysvar::{clock};
use spl_associated_token_account::{instruction::create_associated_token_account, get_associated_token_address};
use crate::common::utils::{mint_bootstrap, sign_send_instructions};
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
use access_protocol::state::BondAccount;

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
    pub async fn new() -> Result<Self, BanksClientError> {
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
        let local_env = prg_test_ctx.banks_client.clone();


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
            .await?;

        //
        // Create authority ACCESS token account
        //
        let ix = create_associated_token_account(
            &prg_test_ctx.payer.pubkey(),
            &prg_test_ctx.payer.pubkey(),
            &mint,
        );
        sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
            .await?;
        let authority_ata = get_associated_token_address(&&prg_test_ctx.payer.pubkey(), &mint);


        Ok(Self {
            program_id,
            prg_test_ctx,
            local_env,
            authority_ata,
            central_state,
            mint,
        })
    }

    pub async fn create_ata_account(&mut self) ->  Result<Keypair, BanksClientError> {
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
            .await?;
        Ok(owner)
    }

    pub async fn mint(&mut self, destination: &Pubkey, amount: u64) -> Result<(), BanksClientError> {
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
    }

    pub async fn create_stake_pool(&mut self, stake_pool_owner: &Pubkey) -> Result<(), BanksClientError>  {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let create_associated_instruction =
            create_associated_token_account(&self.prg_test_ctx.payer.pubkey(), &stake_pool_key, &self.mint);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);
        sign_send_instructions(
            &mut self.prg_test_ctx,
            vec![create_associated_instruction],
            vec![],
        )
            .await?;

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
    }

    pub async fn activate_stake_pool(&mut self, stake_pool_owner: &Pubkey) -> Result<(), BanksClientError> {
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

    pub async fn create_stake_account(&mut self, stake_pool_owner_key: &Pubkey, staker_key: &Pubkey) -> Result<(), BanksClientError>  {
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
    }

    pub async fn sleep(&mut self, duration: u64) -> Result<(), ProgramTestError> {
        self.prg_test_ctx.warp_to_timestamp(
            self.local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp + duration as i64
        ).await
    }

    pub async fn stake(&mut self, stake_pool_owner_key: &Pubkey, staker: &Keypair, token_amount: u64) -> Result<(), BanksClientError> {
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
    }

    pub async fn crank_pool(&mut self, stake_pool_owner_key: &Pubkey) -> Result<(), BanksClientError> {
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
    }

    pub async fn claim_pool_rewards(&mut self, stake_pool_owner: &Keypair) -> Result<(), BanksClientError> {
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
    }

    pub async fn claim_staker_rewards(&mut self, stake_pool_owner: &Pubkey, staker: &Keypair) -> Result<(), BanksClientError>  {
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
    }

    pub async fn unstake(&mut self, stake_pool_owner: &Pubkey, staker: &Keypair, token_amount: u64) -> Result<(), BanksClientError> {
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
        // if error, return
        sign_send_instructions(&mut self.prg_test_ctx, vec![unstake_ix], vec![&staker])
            .await?;

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
    }

    pub async fn staker_stats(&mut self, staker_key: Pubkey) -> Result<StakerStats, BanksClientError> {
        let staker_token_acc = get_associated_token_address(&staker_key, &self.mint);
        let balance = self.local_env.get_packed_account_data::<spl_token::state::Account>(staker_token_acc).await?.amount;
        Ok(StakerStats {
            balance
        })
    }

    pub async fn pool_stats(&mut self, stake_pool_owner: Pubkey) -> Result<PoolOwnerStats, BanksClientError> {
        let stake_pool_owner_token_acc = get_associated_token_address(&stake_pool_owner, &self.mint);
        let balance = self.local_env.get_packed_account_data::<spl_token::state::Account>(stake_pool_owner_token_acc).await?.amount;

        let stake_pool_key = self.get_pool_pda(&stake_pool_owner);
        let stake_pool_associated_token_account = get_associated_token_address(&stake_pool_key, &self.mint);
        let total_pool_staked = self.local_env.get_packed_account_data::<spl_token::state::Account>(stake_pool_associated_token_account).await?.amount;

        Ok(PoolOwnerStats {
            balance,
            total_pool_staked,
        })
    }

    pub async fn create_bond(&mut self, stake_pool_owner: &Pubkey, bond_owner: &Pubkey, bond_amount: u64, bond_maturity: u64) -> Result<(), BanksClientError> {
        let (bond_key, _bond_nonce) =
            BondAccount::create_key(&bond_owner, bond_amount, &self.program_id);

        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let seller_token_acc = get_associated_token_address(&bond_owner, &self.mint);


        let create_bond_ix = create_bond(
            self.program_id,
            create_bond::Accounts {
                stake_pool: &stake_pool_key,
                seller: &self.prg_test_ctx.payer.pubkey(),
                bond_account: &bond_key,
                system_program: &system_program::ID,
                fee_payer: &self.prg_test_ctx.payer.pubkey(),
            },
            create_bond::Params {
                buyer: bond_owner.clone(),
                total_amount_sold: bond_amount,
                seller_token_account: seller_token_acc,
                total_quote_amount: 0,
                quote_mint: Pubkey::default(),
                unlock_period: 1, // todo: make this a parameter
                unlock_amount: bond_amount,
                unlock_start_date: 0,
                seller_index: 0,
            },
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![create_bond_ix], vec![])
            .await
    }
}