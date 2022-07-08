use crate::{error::ProgramError, utils::current_time};
use {
    access_protocol::state::{StakePoolHeaped, Tag, SECONDS_IN_DAY},
    solana_account_decoder::UiAccountEncoding,
    solana_client::{
        rpc_client::RpcClient,
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    },
    solana_program::pubkey::Pubkey,
    solana_sdk::{account::Account, commitment_config::CommitmentConfig},
};

fn filter_pool(x: &(Pubkey, StakePoolHeaped)) -> bool {
    let now = current_time();
    let (_, pool) = x;

    now - pool.header.last_crank_time as u64 > SECONDS_IN_DAY
}

fn deserialize_pool(x: &(Pubkey, Account)) -> (Pubkey, StakePoolHeaped) {
    let (key, acc) = x;
    (*key, StakePoolHeaped::from_buffer(&acc.data[..]))
}

pub fn get_all_pools() -> Result<Vec<Pubkey>, ProgramError> {
    let connection = RpcClient::new(crate::settings::RPC_URL.as_str());

    let bytes = Tag::StakePool as u8 + 1;
    let memcmp = RpcFilterType::Memcmp(Memcmp {
        offset: 0,
        bytes: MemcmpEncodedBytes::Base64(bytes.to_string()),
        encoding: None,
    });

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![memcmp]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig::processed()),
            min_context_slot: None,
        },
        with_context: Some(false),
    };

    let result = connection
        .get_program_accounts_with_config(&access_protocol::ID, config)
        .map_err(|_| ProgramError::Rpc)?;

    let keys: Vec<Pubkey> = result
        .iter()
        .map(deserialize_pool)
        .filter(filter_pool)
        .map(|(key, _)| key)
        .collect();

    Ok(keys)
}
