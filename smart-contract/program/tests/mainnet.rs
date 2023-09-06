use std::fs;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_sdk::account::ReadableAccount;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use access_protocol::instruction::ProgramInstruction::Stake;
use access_protocol::state::StakeAccount;
use crate::common::test_runner::TestRunner;

pub mod common;


fn parse_byte_array(byte_array: &str) -> Vec<u8> {
    byte_array
        .trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .filter_map(|s| s.parse::<u8>().ok())
        .collect()
}

fn load_keypair_from_file(path: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
    let keypair_str = fs::read_to_string(path)?;
    let keypair_bytes = parse_byte_array(&keypair_str);
    let keypair = Keypair::from_bytes(&keypair_bytes)?;
    Ok(keypair)
}

#[test]
fn mainnet() {
    let rpc_url = ""; // todo fill
    let rpc_client = RpcClient::new(rpc_url.to_owned());

    let program_id = Pubkey::from_str("6HW8dXjtiTGkD4jzXs7igdFmZExPpmwUrRN5195xGup").unwrap();

    let (stake_acc_key, stake_nonce) = Pubkey::find_program_address(
        &[
            "stake_account".as_bytes(),
            &Pubkey::from_str("BySCc6DnNEeparG8kYHiWHQ4yi2rAaxJHdmYsJ3r8vXU").unwrap().to_bytes(),
            &Pubkey::from_str("Fxh4hDFHJuTfD3Eq4en36dTk8QvbsSMoTE5Y2hVX3qVt").unwrap().to_bytes(),
        ],
        &program_id,
    );
    println!("stake_acc_key: {:?}", stake_acc_key);

    let acc = rpc_client
        .get_account(&stake_acc_key)
        .unwrap();

    let account = StakeAccount::deserialize(&mut &acc.data[..]).unwrap();
    println!("account: {:?}", account);
}
