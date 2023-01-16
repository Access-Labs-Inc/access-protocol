use std::error::Error;

use borsh::BorshDeserialize;
use solana_program::{pubkey::Pubkey, system_program};

use solana_program_test::{processor, ProgramTest};
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::sysvar::clock;
use solana_test_framework::*;
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};

use access_protocol::{
    entrypoint::process_instruction,
    instruction::{
        activate_stake_pool, admin_mint,
        claim_pool_rewards, claim_rewards,
        crank, create_central_state, create_stake_account,
        create_stake_pool, stake, unstake,
    },
};
use access_protocol::instruction::{change_central_state_authority, change_inflation, change_pool_minimum, change_pool_multiplier, claim_bond, claim_bond_rewards, create_bond, unlock_bond_tokens};
use access_protocol::state::{BondAccount, CentralState, StakeAccount, StakePoolHeader, Tag};

use crate::common::utils::{mint_bootstrap, sign_send_instructions};

pub struct TestRunner {
    pub program_id: Pubkey,
    prg_test_ctx: ProgramTestContext,
    local_env: BanksClient,
    authority_ata: Pubkey,
    central_state: Pubkey,
    mint : Pubkey,
    // hashmap from user pubkey to a bond account
    bond_accounts: std::collections::HashMap<Pubkey, Pubkey>,
    bond_seller: Keypair,
}

pub struct StakerStats {
    pub balance: u64,
}

#[derive(Debug)]
pub struct PoolOwnerStats {
    pub header: StakePoolHeader,
    pub balance: u64,
    pub total_pool_staked: u64,
}

#[derive(Debug)]
pub struct CentralStateStats {
    pub tag: Tag,
    pub daily_inflation: u64,
    pub token_mint: Pubkey,
    pub authority: Pubkey,
    pub total_staked: u64,
}

impl TestRunner {
    pub async fn new(daily_inflation: u64) -> Result<Self, BanksClientError> {
        // Create program and test environment
        let program_id = access_protocol::ID;

        let mut program_test = ProgramTest::default();

        program_test.prefer_bpf(true);
        let mut program_test = ProgramTest::new(
            "access_protocol",
            program_id,
            processor!(process_instruction),
        );
        println!("added access_protocol::ID {:?}", access_protocol::ID);

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);

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


        //
        // Create central state
        //
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
            .await?;

        //
        // Create authority ACCESS token account
        //
        let ix = create_associated_token_account(
            &prg_test_ctx.payer.pubkey(),
            &prg_test_ctx.payer.pubkey(),
            &mint,
            &spl_token::ID,
        );
        sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
            .await?;
        let authority_ata = get_associated_token_address(&prg_test_ctx.payer.pubkey(), &mint);

        // Create bond seller
        let bond_seller = Keypair::new();
        let create_ata_stake_pool_owner_ix = create_associated_token_account(
            &prg_test_ctx.payer.pubkey(),
            &bond_seller.pubkey(),
            &mint,
            &spl_token::ID,
        );
        sign_send_instructions(
            &mut prg_test_ctx,
            vec![create_ata_stake_pool_owner_ix],
            vec![],
        )
            .await?;

        Ok(Self {
            program_id,
            prg_test_ctx,
            local_env,
            authority_ata,
            central_state,
            mint,
            bond_accounts: std::collections::HashMap::new(),
            bond_seller,
        })
    }

    pub async fn create_ata_account(&mut self) ->  Result<Keypair, BanksClientError> {
        let owner = Keypair::new();
        let create_ata_stake_pool_owner_ix = create_associated_token_account(
            &self.prg_test_ctx.payer.pubkey(),
            &owner.pubkey(),
            &self.mint,
            &spl_token::ID,
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
        let destination_ata = get_associated_token_address(destination, &self.mint);
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

    pub async fn create_stake_pool(&mut self, stake_pool_owner: &Pubkey, minimum_stake_amount: u64) -> Result<(), BanksClientError>  {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let create_associated_instruction =
            create_associated_token_account(&self.prg_test_ctx.payer.pubkey(), &stake_pool_key, &self.mint, &spl_token::ID);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);
        let create_ata_pool_vault_ix = create_associated_token_account(
            &self.prg_test_ctx.payer.pubkey(),
            &stake_pool_key,
            &self.mint,
            &spl_token::ID,
        );
        sign_send_instructions(
            &mut self.prg_test_ctx,
            vec![create_ata_pool_vault_ix],
            vec![],
        )
            .await?;
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
                minimum_stake_amount,
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
        let (stake_acc_key, stake_nonce) = self.get_stake_account_pda(&stake_pool_key, staker_key);
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
        // get the staker's bond from the hash map if it exists
        let staker_bond: Option<&Pubkey> = self.bond_accounts.get(&staker_key);

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
                bond_account: staker_bond,
            },
            stake::Params {
                amount: token_amount,
            },
        );
        sign_send_instructions(&mut self.prg_test_ctx, vec![stake_ix], vec![staker])
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
            vec![stake_pool_owner],
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

        sign_send_instructions(&mut self.prg_test_ctx, vec![claim_ix], vec![staker])
            .await
    }

    pub async fn unstake(&mut self, stake_pool_owner: &Pubkey, staker: &Keypair, token_amount: u64) -> Result<(), BanksClientError> {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let (stake_acc_key, _) = self.get_stake_account_pda(&stake_pool_key, &staker.pubkey());
        let staker_token_acc = get_associated_token_address(&staker.pubkey(), &self.mint);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);

        // get the staker's bond from the hash map if it exists
        let staker_bond: Option<&Pubkey> = self.bond_accounts.get(&staker.pubkey());

        // Request Unstake
        let unstake_ix = unstake(
            self.program_id,
            unstake::Accounts {
                stake_account: &stake_acc_key,
                stake_pool: &stake_pool_key,
                owner: &staker.pubkey(),
                destination_token: &staker_token_acc,
                spl_token_program: &spl_token::ID,
                central_state_account: &self.central_state,
                vault: &pool_vault,
                bond_account: staker_bond,
            },
            unstake::Params {
                amount: token_amount,
            },
        );
        // if error, return
        sign_send_instructions(&mut self.prg_test_ctx, vec![unstake_ix], vec![staker])
            .await
    }

    pub async fn staker_stats(&mut self, staker_key: Pubkey) -> Result<StakerStats, BanksClientError> {
        let staker_token_acc = get_associated_token_address(&staker_key, &self.mint);
        let balance = self.local_env.get_packed_account_data::<spl_token::state::Account>(staker_token_acc).await?.amount;
        Ok(StakerStats {
            balance
        })
    }


    pub async fn stake_account_stats(&mut self, staker: Pubkey, stake_pool_owner: Pubkey) -> Result<StakeAccount, BanksClientError> {
        let stake_pool_key = self.get_pool_pda(&stake_pool_owner);
        let (stake_acc_key, _) = self.get_stake_account_pda(&stake_pool_key, &staker);

        let acc = self.prg_test_ctx
            .banks_client
            .get_account(stake_acc_key)
            .await
            .unwrap()
            .unwrap();
        let account = StakeAccount::deserialize(&mut &acc.data[..])?;
        Ok(account)
    }

    pub async fn pool_stats(&mut self, stake_pool_owner: Pubkey) -> Result<PoolOwnerStats, BanksClientError> {
        let stake_pool_owner_token_acc = get_associated_token_address(&stake_pool_owner, &self.mint);
        let balance = self.local_env.get_packed_account_data::<spl_token::state::Account>(stake_pool_owner_token_acc).await?.amount;

        let stake_pool_key = self.get_pool_pda(&stake_pool_owner);
        let stake_pool_associated_token_account = get_associated_token_address(&stake_pool_key, &self.mint);
        let total_pool_staked = self.local_env.get_packed_account_data::<spl_token::state::Account>(stake_pool_associated_token_account).await?.amount;

        let acc = self.prg_test_ctx
            .banks_client
            .get_account(stake_pool_key)
            .await
            .unwrap()
            .unwrap();
        let pool_header = StakePoolHeader::deserialize(&mut &acc.data[..])?;

        Ok(PoolOwnerStats {
            header: pool_header,
            balance,
            total_pool_staked,
        })
    }

    // bond stats
    pub async fn bond_stats(&mut self, bond_owner: Pubkey, stake_pool_owner: Pubkey, original_bond_amount: u64) -> Result<BondAccount, BanksClientError> {
        let _stake_pool_key = self.get_pool_pda(&stake_pool_owner);
        let (bond_key, _) =
            BondAccount::create_key(&bond_owner, original_bond_amount, &self.program_id);

        let acc = self.prg_test_ctx
            .banks_client
            .get_account(bond_key)
            .await
            .unwrap()
            .unwrap();
        let bond_account = BondAccount::deserialize(&mut &acc.data[..])?;
        Ok(bond_account)
    }

    pub async fn central_state_stats(&mut self) -> Result<CentralState, Box<dyn Error>> {
        let acc = self.prg_test_ctx
            .banks_client
            .get_account(self.central_state)
            .await
            .unwrap()
            .unwrap();
        let cs = CentralState::deserialize(&mut &acc.data[..])?;
        Ok(cs)
    }

    pub async fn create_bond(&mut self, stake_pool_owner: &Pubkey, bond_owner: &Pubkey, total_amount: u64, payout_count: u64, unlock_after: i64, unlock_period: i64) -> Result<(), BanksClientError> {
        let (bond_key, _bond_nonce) =
            BondAccount::create_key(bond_owner, total_amount, &self.program_id);

        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let seller_token_account = get_associated_token_address(&self.bond_seller.pubkey(), &self.mint);
        self.mint(&self.bond_seller.pubkey(), total_amount).await?;
        let current_time = self.local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp;

        let create_bond_ix = create_bond(
            self.program_id,
            create_bond::Accounts {
                seller: &self.bond_seller.pubkey(),
                bond_account: &bond_key,
                stake_pool: &stake_pool_key, // OK
                system_program: &system_program::ID, // OK
                fee_payer: &self.prg_test_ctx.payer.pubkey(),
            },
            create_bond::Params {
                buyer: *bond_owner,
                total_amount_sold: total_amount,
                seller_token_account,
                total_quote_amount: 0,
                quote_mint: self.mint,
                unlock_period,
                unlock_amount: total_amount / payout_count,
                unlock_start_date: current_time + unlock_after,
                seller_index: 0,
            },
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![create_bond_ix], vec![&self.bond_seller])
            .await?;

        // add bond account to the map
        self.bond_accounts.insert(*bond_owner, bond_key);
        Ok(())
    }

    pub async fn claim_bond(&mut self, stake_pool_owner: &Pubkey, bond_owner: &Pubkey) -> Result<(), BanksClientError> {
        let bond_key = *self.bond_accounts.get(&bond_owner).unwrap();
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let seller_token_acc = get_associated_token_address(&self.bond_seller.pubkey(), &self.mint);
        let bond_owner_ata = get_associated_token_address(&bond_owner, &self.mint);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);

        let mut claim_bond_ix = claim_bond(
            self.program_id,
            claim_bond::Accounts {
                bond_account: &bond_key,
                buyer: &bond_owner,
                quote_token_source: &bond_owner_ata,
                quote_token_destination: &seller_token_acc,
                stake_pool: &stake_pool_key,
                access_mint: &self.mint,
                pool_vault: &pool_vault,
                central_state: &self.central_state,
                spl_token_program: &spl_token::ID,
            },
            claim_bond::Params {},
        );
        claim_bond_ix.accounts[1].is_signer = false;

    println!("claiming bond");
        sign_send_instructions(&mut self.prg_test_ctx, vec![claim_bond_ix], vec![])
            .await
    }

    pub async fn create_bond_with_quote(&mut self, stake_pool_owner: &Pubkey, bond_owner: &Pubkey, bond_amount: u64, quote_amount: u64, unlock_after: i64) -> Result<(), BanksClientError> {
        let (bond_key, _bond_nonce) =
            BondAccount::create_key(bond_owner, bond_amount, &self.program_id);

        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let seller_token_account = get_associated_token_address(&self.bond_seller.pubkey(), &self.mint);
        self.mint(&self.bond_seller.pubkey(), bond_amount).await?;
        let current_time = self.local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp;

        let create_bond_ix = create_bond(
            self.program_id,
            create_bond::Accounts {
                seller: &self.bond_seller.pubkey(),
                bond_account: &bond_key,
                stake_pool: &stake_pool_key, // OK
                system_program: &system_program::ID, // OK
                fee_payer: &self.prg_test_ctx.payer.pubkey(),
            },
            create_bond::Params {
                buyer: *bond_owner,
                total_amount_sold: bond_amount,
                seller_token_account,
                total_quote_amount: quote_amount,
                quote_mint: self.mint,
                unlock_period: 1, // todo: make this a parameter
                unlock_amount: bond_amount,
                unlock_start_date: current_time + unlock_after,
                seller_index: 0,
            },
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![create_bond_ix], vec![&self.bond_seller])
            .await?;

        // add bond account to the map
        self.bond_accounts.insert(*bond_owner, bond_key);
        Ok(())
    }

    pub async fn claim_bond_with_quote(&mut self, stake_pool_owner: &Pubkey, bond_owner: &Keypair) -> Result<(), BanksClientError> {
        let bond_key = *self.bond_accounts.get(&bond_owner.pubkey()).unwrap();
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let seller_token_acc = get_associated_token_address(&self.bond_seller.pubkey(), &self.mint);
        let bond_owner_ata = get_associated_token_address(&bond_owner.pubkey(), &self.mint);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);

        let claim_bond_ix = claim_bond(
            self.program_id,
            claim_bond::Accounts {
                bond_account: &bond_key,
                buyer: &bond_owner.pubkey(),
                quote_token_source: &bond_owner_ata,
                quote_token_destination: &seller_token_acc,
                stake_pool: &stake_pool_key,
                access_mint: &self.mint,
                pool_vault: &pool_vault,
                central_state: &self.central_state,
                spl_token_program: &spl_token::ID,
            },
            claim_bond::Params {},
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![claim_bond_ix], vec![bond_owner])
            .await
    }

    pub async fn unlock_bond(&mut self, stake_pool_owner: &Pubkey, bond_owner: &Keypair) -> Result<(), BanksClientError> {
        let bond_key = *self.bond_accounts.get(&bond_owner.pubkey()).unwrap();
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let pool_vault = get_associated_token_address(&stake_pool_key, &self.mint);
        let bond_owner_ata = get_associated_token_address(&bond_owner.pubkey(), &self.mint);

        let unlock_ix = unlock_bond_tokens(
            self.program_id,
            unlock_bond_tokens::Accounts {
                bond_account: &bond_key,
                bond_owner: &bond_owner.pubkey(),
                mint: &self.mint,
                access_token_destination: &bond_owner_ata,
                central_state: &self.central_state,
                spl_token_program: &spl_token::ID,
                stake_pool: &stake_pool_key,
                pool_vault: &pool_vault,
            },
            unlock_bond_tokens::Params {},
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![unlock_ix], vec![bond_owner])
            .await
    }

    pub async fn claim_bond_rewards(&mut self, stake_pool_owner: &Pubkey, bond_owner: &Keypair) -> Result<(), BanksClientError> {
        let stake_pool_key = self.get_pool_pda(stake_pool_owner);
        let bond_key = self.bond_accounts.get(&bond_owner.pubkey()).unwrap();
        let seller_token_acc = get_associated_token_address(&bond_owner.pubkey(), &self.mint);

        let claim_bond_rewards_ix = claim_bond_rewards(
            self.program_id,
            claim_bond_rewards::Accounts {
                stake_pool: &stake_pool_key,
                bond_account: bond_key,
                bond_owner: &bond_owner.pubkey(),
                rewards_destination: &seller_token_acc,
                central_state: &self.central_state,
                mint: &self.mint,
                spl_token_program: &spl_token::ID,
            },
            claim_bond_rewards::Params {},
            false,
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![claim_bond_rewards_ix], vec![])
            .await
    }

    pub fn get_authority(&self) -> Pubkey {
        self.prg_test_ctx.payer.pubkey()
    }

    pub fn get_mint(&self) -> Pubkey {
        self.mint
    }

    pub async fn get_current_time(&mut self) -> i64 {
        self.local_env.get_sysvar::<clock::Clock>().await.unwrap().unix_timestamp
    }

    pub fn get_bond_seller(&self) -> Pubkey {
        self.bond_seller.pubkey()
    }

    pub fn get_bond_seller_ata(&self) -> Pubkey {
        get_associated_token_address(&self.bond_seller.pubkey(), &self.mint)
    }

    pub async fn change_pool_minimum(&mut self, stake_pool_owner: &Keypair, new_minimum: u64) -> Result<(), BanksClientError> {
        let stake_pool_key = self.get_pool_pda(&stake_pool_owner.pubkey());
        let change_min_ix = change_pool_minimum(
            self.program_id,
            change_pool_minimum::Accounts {
                stake_pool: &stake_pool_key,
                stake_pool_owner: &stake_pool_owner.pubkey(),
            },
            change_pool_minimum::Params {
                new_minimum,
            },
        );

        sign_send_instructions(
            &mut self.prg_test_ctx,
            vec![change_min_ix],
            vec![stake_pool_owner],
        )
            .await
    }

    pub async fn change_pool_multiplier(&mut self, stake_pool_owner: &Keypair, new_multiplier: u64) -> Result<(), BanksClientError> {
        let stake_pool_key = self.get_pool_pda(&stake_pool_owner.pubkey());
        let change_min_ix = change_pool_multiplier(
            self.program_id,
            change_pool_multiplier::Accounts {
                stake_pool: &stake_pool_key,
                stake_pool_owner: &stake_pool_owner.pubkey(),
            },
            change_pool_multiplier::Params {
                new_multiplier,
            },
        );

        sign_send_instructions(
            &mut self.prg_test_ctx,
            vec![change_min_ix],
            vec![stake_pool_owner],
        )
            .await
    }

    pub async fn change_inflation(&mut self, new_inflation: u64) -> Result<(), BanksClientError> {
        let change_inflation_ix = change_inflation(
            self.program_id,
            change_inflation::Accounts {
                central_state: &self.central_state,
                authority: &self.prg_test_ctx.payer.pubkey(),
            },
            change_inflation::Params {
                daily_inflation: new_inflation,
            },
        );

        sign_send_instructions(&mut self.prg_test_ctx, vec![change_inflation_ix], vec![])
            .await
    }

    pub async fn change_central_state_authority(&mut self, new_authority: &Keypair) -> Result<(), BanksClientError> {
        let ix = change_central_state_authority(
            self.program_id,
            change_central_state_authority::Accounts {
                central_state: &self.central_state,
                authority: &self.prg_test_ctx.payer.pubkey(),
            },
            change_central_state_authority::Params {
                new_authority: new_authority.pubkey(),
            },
        );
        sign_send_instructions(&mut self.prg_test_ctx, vec![ix], vec![])
            .await?;

        let authority_ata = get_associated_token_address(&self.prg_test_ctx.payer.pubkey(), &self.mint);
        self.authority_ata = authority_ata;
        Ok(())
    }
}