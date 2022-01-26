use crate::errors::AccessError;
use access_protocol::state::StakeAccount;
use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey, pubkey::Pubkey};

const ENDPOINT: &str = "https://devnet.solana.com"; // TODO change
const STAKE_POOL: Pubkey = pubkey!("6gbWg2YqjCwzsBYwepCxeaJdy5x4BmXRAhSN9QoMsrbH"); // TODO change
const PROGRAM_ID: Pubkey = pubkey!("6gbWg2YqjCwzsBYwepCxeaJdy5x4BmXRAhSN9QoMsrbH"); // TODO change
const STAKE_POOL_MIN: u64 = 1_000_000; // TODO change

pub async fn check_stake_account(staker: Pubkey) -> Result<(), AccessError> {
    let (stake_key, _) = StakeAccount::find_key(&staker, &STAKE_POOL, &PROGRAM_ID);
    let connection = RpcClient::new(ENDPOINT.to_owned());
    let account_data = connection
        .get_account_data(&stake_key)
        .map_err(|_| AccessError::RpcError)?;
    let stake_account =
        StakeAccount::deserialize(&mut &account_data[..]).map_err(|_| AccessError::BorshError)?;

    if stake_account.stake_amount < STAKE_POOL_MIN {
        return Err(AccessError::NotEnoughStake);
    }

    Ok(())
}
