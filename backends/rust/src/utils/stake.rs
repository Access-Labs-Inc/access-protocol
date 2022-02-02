use crate::errors::AccessError;
use {
    access_protocol::state::StakeAccount,
    borsh::BorshDeserialize,
    solana_client::rpc_client::RpcClient,
    solana_program::{pubkey, pubkey::Pubkey},
};

const ENDPOINT: &str = "https://api.devnet.solana.com"; // TODO change
const STAKE_POOL: Pubkey = pubkey!("Hs6emyaDnMSxJmGxnHhSmucJh1Q9jSysuKJ5yycWoUuC"); // TODO change
const PROGRAM_ID: Pubkey = pubkey!("2ZsWiVGXzL4kgMDtSfeEJSV27fBnMptrdcNKKZygUoB8"); // TODO change

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
