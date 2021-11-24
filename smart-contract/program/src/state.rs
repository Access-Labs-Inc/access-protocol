pub const SECONDS_IN_DAY: u64 = 3600 * 24;

pub struct StakePool {
    // Total amount staked in the pool
    pub total_staked: u64,
    // Last unix timestamp when rewards were paid to the pool owner
    // through a permissionless crank
    pub last_crank_time: i64,
    // Owner of the stake pool and the rewards_destination token account
    pub owner: [u8; 32],
    // Address to which rewards are sent
    pub rewards_destination: [u8; 32],
    // Stake pool nonce
    pub nonce: u8,
    // Name of the stake pool (used for PDA derivation)
    pub name: String,
}

pub struct StakeAccount {
    // Owner of the stake account
    pub owner: [u8; 32],
    // Amount staked in the account
    pub stake_amount: u64,
}

pub struct CentralState {
    // Central state nonce
    pub signer_nonce: u8,
    // Daily inflation in token amount, inflation is paid from 
    // the reserve owned by the central state
    pub daily_inflation: u64, 
}
