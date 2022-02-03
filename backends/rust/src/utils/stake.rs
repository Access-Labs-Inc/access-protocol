use crate::errors::AccessError;
use {
    access_protocol::state::StakeAccount, borsh::BorshDeserialize, dotenv,
    lazy_static::lazy_static, solana_client::rpc_client::RpcClient, solana_program::pubkey::Pubkey,
    std::str::FromStr,
};

lazy_static! {
    pub static ref ENDPOINT: String = dotenv::var("RPC_URL").unwrap();
    pub static ref STAKE_POOL: Pubkey =
        Pubkey::from_str(&dotenv::var("STAKE_POOL_KEY").unwrap().as_str()).unwrap();
    pub static ref PROGRAM_ID: Pubkey =
        Pubkey::from_str(&dotenv::var("PROGRAM_ID").unwrap().as_str()).unwrap();
}

pub async fn check_stake_account(staker: Pubkey) -> Result<(), AccessError> {
    let stake_key = StakeAccount::find_key(&staker, &STAKE_POOL, &PROGRAM_ID).0;
    println!("{}", stake_key.to_string());
    let connection = RpcClient::new(ENDPOINT.to_owned());
    let account_data = connection
        .get_account_data(&stake_key)
        .map_err(|_| AccessError::RpcError)?;
    let stake_account =
        StakeAccount::deserialize(&mut &account_data[..]).map_err(|_| AccessError::BorshError)?;

    if stake_account.stake_amount < stake_account.pool_minimum_at_creation {
        return Err(AccessError::NotEnoughStake);
    }

    Ok(())
}
