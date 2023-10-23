use {lazy_static::lazy_static, solana_sdk::signature::Keypair, solana_program::pubkey::Pubkey};
use std::str::FromStr;
lazy_static! {
    pub static ref RPC_URL: String = dotenv::var("RPC_URL").unwrap();
    pub static ref PAYER: Keypair = Keypair::from_base58_string(&dotenv::var("PAYER").unwrap());
    pub static ref PROGRAM_ID: Pubkey = Pubkey::from_str(&dotenv::var("PROGRAM_ID").unwrap()).unwrap();
    pub static ref MINT: Pubkey = Pubkey::from_str(&dotenv::var("MINT").unwrap()).unwrap();
}

pub const CYCLE_INTERVAL: u64 = 60 * 60;
