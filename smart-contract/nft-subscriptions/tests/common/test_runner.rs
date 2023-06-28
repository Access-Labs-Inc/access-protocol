use std::error::Error;
use std::str::FromStr;

use borsh::BorshDeserialize;
use solana_program::{pubkey::Pubkey, system_program};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::sysvar::clock;
use solana_test_framework::*;
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};

use access_protocol_nft::{
    entrypoint::process_instruction,
    instruction::mint_subscription,
};

use crate::common::utils::{mint_bootstrap, sign_send_instructions};

pub struct TestRunner {
    pub program_id: Pubkey,
    prg_test_ctx: ProgramTestContext,
    local_env: BanksClient,
    // hashmap from user pubkey to a bond account
}

impl TestRunner {
    pub async fn new() -> Result<Self, BanksClientError> {
        // Create program and test environment
        let program_id = access_protocol_nft::ID;

        let mut program_test = ProgramTest::default();

        program_test.prefer_bpf(true);
        let mut program_test = ProgramTest::new(
            "access_protocol_nft",
            program_id,
            processor!(process_instruction),
        );
        println!("added access_protocol::ID {:?}", access_protocol_nft::ID);

        let mut prg_test_ctx = program_test.start_with_context().await;
        let local_env = prg_test_ctx.banks_client.clone();

        Ok(Self {
            program_id,
            prg_test_ctx,
            local_env,
        })
    }

    pub async fn mint_subscription_nft(&mut self,
                                       owner: &Keypair,
    ) -> Result<Pubkey, BanksClientError> {
        let mint = Keypair::new();
        println!("mint: {:?}", mint);
        let token_account = get_associated_token_address(&owner.pubkey(), &mint.pubkey());
        println!("token_account: {:?}", token_account);
        let mint_subscription_ix = mint_subscription(
            self.program_id,
            mint_subscription::Accounts {
                fee_payer: &self.prg_test_ctx.payer.pubkey(),
                mint: &mint.pubkey(),
                token_account: &token_account,
                mint_authority: &owner.pubkey(),
                rent: &Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap(),
                system_program: &system_program::ID,
                token_program: &Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
                associated_token_program: &Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
            },
            mint_subscription::Params {},
        );
        println!("mint_subscription_ix: {:?}", mint_subscription_ix);
        sign_send_instructions(&mut self.prg_test_ctx, vec![mint_subscription_ix], vec![owner, &mint])
            .await?;
        Ok(mint.pubkey())
    }
}