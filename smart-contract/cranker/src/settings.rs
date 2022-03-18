use {lazy_static::lazy_static, solana_sdk::signature::Keypair};

lazy_static! {
    pub static ref RPC_URL: String = dotenv::var("RPC_URL").unwrap();
    pub static ref PAYER: Keypair = Keypair::from_base58_string(&dotenv::var("PAYER").unwrap());
}

pub const CYCLE_INTERVAL: u64 = 60 * 60;
