use std::fs;
use std::str::FromStr;

use borsh::BorshDeserialize;
use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;

use solana_sdk::signature::{Keypair, read_keypair_file, Signature, Signer};
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;

use access_protocol::instruction::{activate_stake_pool, admin_mint, admin_setup_fee_split, crank, create_stake_account, create_stake_pool, stake};
use access_protocol::state::{CentralStateV2, StakePool};
use access_protocol::state::StakeAccount;

pub mod common;


#[test]
fn devnet() {
    let program_keypair = must_load_keypair("../../scripts/artifacts/program.json");
    let authority_keypair = must_load_keypair("../../scripts/artifacts/authority.json");
    let token_bank_keypair = must_load_keypair("../../scripts/artifacts/token_bank.json");
    let pool_keypair = must_load_keypair("../../scripts/artifacts/pool.json");
    let token_mint = Pubkey::from_str(
        fs::read_to_string("../../scripts/artifacts/mint_address.txt").unwrap().as_str()
    ).unwrap();

    let program_id = program_keypair.pubkey();
    println!("program id: {:?}", program_id);
    println!("authority: {:?}", authority_keypair.pubkey());
    println!("token bank: {:?}", token_bank_keypair.pubkey());
    println!("pool: {:?}", pool_keypair.pubkey());
    println!("token mint: {:?}", token_mint);

    let rpc_url = "https://api.devnet.solana.com".to_owned();
    let rpc_client = RpcClient::new(rpc_url);

    let (central_state_key, _) = CentralStateV2::find_key(&program_id);
    println!("central state key: {:?}", central_state_key);

    let acc = rpc_client.get_account(&central_state_key).unwrap();

    println!("account data len: {}", acc.data.len());
    println!("account owner: {}", acc.owner);
    let account = CentralStateV2::deserialize(&mut &acc.data[..]).unwrap();
    account.recipients.iter().for_each(|r| {
        println!("{}: {}", r.owner, r.percentage);
    });
    println!("account: {:?}", account);

    // not checking the result here as it will fail if the account already exists
    let _ = create_ata(&rpc_client, central_state_key, &authority_keypair, token_mint);

    let token_bank = token_bank_keypair.pubkey();
    let token_bank_ata = get_associated_token_address(&token_bank, &token_mint);

    // not checking the result here as it will fail if the account already exists
    let _ = create_ata(&rpc_client, token_bank, &authority_keypair, token_mint);

    // ------------------------------------
    // MINT
    // ------------------------------------

    let admin_mint_ix = admin_mint(
        program_id,
        admin_mint::Accounts {
            authority: &authority_keypair.pubkey(),
            mint: &token_mint,
            access_token_destination: &token_bank_ata,
            central_state: &central_state_key,
            spl_token_program: &spl_token::ID,
        },
        admin_mint::Params { amount: 10_200_000_000 },
    );

    let signers = vec![&authority_keypair];
    let mut transaction = Transaction::new_with_payer(&[admin_mint_ix], Some(&authority_keypair.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
    transaction.sign(&signers, recent_blockhash);
    let result = rpc_client.send_and_confirm_transaction(&transaction);
    println!("Mint result: {:?}", result);

    // ------------------------------------
    // CREATE A POOL
    // ------------------------------------
    let (pool_key, _) = StakePool::find_key(&pool_keypair.pubkey(), &program_id);

    'pool_creation: {
        // break 'pool_creation; // comment out to create the pool

        // not checking the result here as it will fail if the account already exists
        let _ = create_ata(&rpc_client, pool_key, &authority_keypair, token_mint);
        let pool_vault = get_associated_token_address(&pool_key, &token_mint);
        println!("Pool vault: {:?}", pool_vault);

        let create_stake_pool_ix = create_stake_pool(
            program_id,
            create_stake_pool::Accounts {
                stake_pool_account: &pool_key,
                system_program: &system_program::ID,
                fee_payer: &authority_keypair.pubkey(),
                vault: &pool_vault,
                central_state: &central_state_key,
            },
            create_stake_pool::Params {
                owner: pool_keypair.pubkey(),
                minimum_stake_amount: 50_000_000,
            },
        );

        let signers = vec![&authority_keypair];
        let mut transaction = Transaction::new_with_payer(&[create_stake_pool_ix], Some(&authority_keypair.pubkey()));
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction.sign(&signers, recent_blockhash);
        let result = rpc_client.send_and_confirm_transaction(&transaction);
        println!("Pool creation result: {:?}", result);

        let result = activate_pool(&rpc_client, program_id, pool_key, &authority_keypair);
        println!("Pool activation result: {:?}", result);

        let result = create_stake_acc(&rpc_client, program_id, pool_key, token_bank, &authority_keypair);
        println!("Stake account creation result: {:?}", result);
    }
    // this can fail and we don't care
    let _ = crank_pool(&rpc_client, program_id, pool_key, &authority_keypair);

    // ------------------------------------
    // STAKE
    // ------------------------------------

    'stake: {
        // break 'stake; // comment out to stake
        let result = lock(
            &rpc_client,
            program_id,
            &token_bank_keypair,
            pool_key,
            token_mint,
            10_000_000_000,
        );
        println!("Stake result: {:?}", result);
    }
    return;
    // ------------------------------------
    // DISTRIBUTE FEES
    // ------------------------------------
    'distribute_fees: {
        // break 'distribute_fees; // comment out to distribute fees
        let result = distribute_fees(&rpc_client, program_id, &authority_keypair);
        println!("Distribute fees result: {:?}", result);
    }

    // ------------------------------------
    // SET FEE RECIPIENTS
    // ------------------------------------
    'set_fee_recipients: {
        // break 'set_fee_recipients; // comment out to set fee recipients
        let result = set_fee_recipients(&rpc_client, program_id, &authority_keypair, vec![
            access_protocol::state::FeeRecipient {
                owner: token_bank,
                percentage: 30,
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 3
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 2
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 2
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 2
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 2
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 2
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 2
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 2
            },
            access_protocol::state::FeeRecipient {
                owner: Keypair::new().pubkey(),
                percentage: 2
            },
        ]);
        println!("Set fee recipients result: {:?}", result);
    }

    // ------------------------------------
    // FINAL PRINT OF THE CENTRAL STATE
    // ------------------------------------
    let acc = rpc_client.get_account(&central_state_key).unwrap();
    let account = CentralStateV2::deserialize(&mut &acc.data[..]).unwrap();
    println!("account: {:?}", account);
}

fn must_load_keypair(file_path: &str) -> Keypair{
    Keypair::from_bytes(
        &read_keypair_file(file_path).unwrap().to_bytes()
    ).unwrap()
}


fn activate_pool(
    connection: &RpcClient,
    program_id: Pubkey,
    pool: Pubkey,
    fee_payer: &Keypair,
) -> Result<Signature, ClientError> {
    let (central_state_key, _) = CentralStateV2::find_key(&program_id);
    let activate_stake_pool_ix = activate_stake_pool(
        program_id,
        activate_stake_pool::Accounts {
            stake_pool: &pool,
            central_state: &central_state_key,
        },
        activate_stake_pool::Params {},
    );

    let mut transaction = Transaction::new_with_payer(&[activate_stake_pool_ix], Some(&fee_payer.pubkey()));
    let recent_blockhash = connection.get_latest_blockhash().unwrap();
    transaction.sign(&vec![fee_payer], recent_blockhash);
    connection.send_and_confirm_transaction(&transaction)
}

fn create_stake_acc(
    connection: &RpcClient,
    program_id: Pubkey,
    pool: Pubkey,
    staker: Pubkey,
    fee_payer: &Keypair,
) -> Result<Signature, ClientError> {
    let (stake_acc_key, bump_seed) = StakeAccount::find_key(&staker, &pool, &program_id);
    let (central_state_key, _) = CentralStateV2::find_key(&program_id);
    let create_stake_account_ix = create_stake_account(
        program_id,
        create_stake_account::Accounts {
            stake_account: &stake_acc_key,
            system_program: &system_program::ID,
            fee_payer: &fee_payer.pubkey(),
            stake_pool: &pool,
            central_state: &central_state_key,
        },
        create_stake_account::Params {
            nonce: bump_seed,
            owner: staker,
        },
    );

    let mut transaction = Transaction::new_with_payer(&[create_stake_account_ix], Some(&fee_payer.pubkey()));
    let recent_blockhash = connection.get_latest_blockhash().unwrap();
    transaction.sign(&vec![fee_payer], recent_blockhash);
    connection.send_and_confirm_transaction(&transaction)
}

fn lock(
    connection: &RpcClient,
    program_id: Pubkey,
    staker: &Keypair,
    pool: Pubkey,
    token_mint: Pubkey,
    amount: u64,
) -> Result<Signature, ClientError> {
    let (stake_acc_key, _) = StakeAccount::find_key(&staker.pubkey(), &pool, &program_id);
    let pool_vault = get_associated_token_address(&pool, &token_mint);
    let (central_state, _) = CentralStateV2::find_key(&program_id);
    let central_state_vault = get_associated_token_address(&central_state, &token_mint);
    let acc = connection.get_account(&central_state_vault).unwrap();
    println!("account owner: {}", acc.owner);
    println!("program id: {}", program_id);
    let stake_ix = stake(
        program_id,
        stake::Accounts {
            stake_account: &stake_acc_key,
            stake_pool: &pool,
            owner: &staker.pubkey(),
            source_token: &get_associated_token_address(&staker.pubkey(), &token_mint),
            spl_token_program: &spl_token::ID,
            vault: &pool_vault,
            central_state: &central_state,
            central_state_vault: &central_state_vault,
        },
        stake::Params {
            amount,
        },
    );

    let mut transaction = Transaction::new_with_payer(&[stake_ix], Some(&staker.pubkey()));
    let recent_blockhash = connection.get_latest_blockhash().unwrap();
    transaction.sign(&vec![staker], recent_blockhash);
    connection.send_and_confirm_transaction(&transaction)
}

fn distribute_fees(
    connection: &RpcClient,
    program_id: Pubkey,
    fee_payer: &Keypair,
) -> Result<Signature, ClientError> {
    let (central_state_key, _) = CentralStateV2::find_key(&program_id);
    let acc = connection.get_account(&central_state_key).unwrap();
    let account = CentralStateV2::deserialize(&mut &acc.data[..]).unwrap();
    let recipient_pubkeys: Vec<Pubkey> = account
        .recipients
        .iter()
        .map(|r| get_associated_token_address(&r.owner, &account.token_mint))
        .collect();
    let central_state_vault = get_associated_token_address(&central_state_key, &account.token_mint);

    let distribute_fees_ix = access_protocol::instruction::distribute_fees(
        program_id,
        access_protocol::instruction::distribute_fees::Accounts {
            central_state: &central_state_key,
            central_state_vault: &central_state_vault,
            spl_token_program: &spl_token::ID,
            mint: &account.token_mint,
            token_accounts: recipient_pubkeys.leak(),
        },
        access_protocol::instruction::distribute_fees::Params {},
    );

    let mut transaction = Transaction::new_with_payer(&[distribute_fees_ix], Some(&fee_payer.pubkey()));
    let recent_blockhash = connection.get_latest_blockhash().unwrap();
    transaction.sign(&vec![fee_payer], recent_blockhash);
    connection.send_and_confirm_transaction(&transaction)
}

fn set_fee_recipients(
    connection: &RpcClient,
    program_id: Pubkey,
    authority: &Keypair,
    recipients: Vec<access_protocol::state::FeeRecipient>,
) -> Result<Signature, ClientError> {
    let central_state_key = CentralStateV2::find_key(&program_id).0;
    let admin_setup_fee_split_ix = admin_setup_fee_split(
        program_id,
        admin_setup_fee_split::Accounts {
            authority: &authority.pubkey(),
            central_state: &central_state_key,
            system_program: &system_program::ID,
        },
        admin_setup_fee_split::Params { recipients },
    );

    let mut transaction = Transaction::new_with_payer(&[admin_setup_fee_split_ix], Some(&authority.pubkey()));
    let recent_blockhash = connection.get_latest_blockhash().unwrap();
    transaction.sign(&vec![authority], recent_blockhash);
    connection.send_and_confirm_transaction(&transaction)
}

fn create_ata(
    connection: &RpcClient,
    owner: Pubkey,
    fee_payer: &Keypair,
    mint: Pubkey,
) -> Result<Signature, ClientError> {
    let ix = create_associated_token_account(
        &fee_payer.pubkey(),
        &owner,
        &mint,
        &spl_token::ID,
    );

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&fee_payer.pubkey()));
    let recent_blockhash = connection.get_latest_blockhash().unwrap();
    transaction.sign(&vec![fee_payer], recent_blockhash);
    // not checking the result here as it will fail if the account already exists
    connection.send_and_confirm_transaction(&transaction)
}

pub fn crank_pool(
    connection: &RpcClient,
    program_id: Pubkey,
    pool: Pubkey,
    fee_payer: &Keypair,
) -> Result<Signature, ClientError> {
    let central_state_key = CentralStateV2::find_key(&program_id).0;
    let crank_ix = crank(
        program_id,
        crank::Accounts {
            stake_pool: &pool,
            central_state: &central_state_key,
        },
        crank::Params {},
    );

    let mut transaction = Transaction::new_with_payer(&[crank_ix], Some(&fee_payer.pubkey()));
    let recent_blockhash = connection.get_latest_blockhash().unwrap();
    transaction.sign(&vec![fee_payer], recent_blockhash);
    connection.send_and_confirm_transaction(&transaction)
}