use crate::{error::ProgramError, utils::current_time};
use {
    access_protocol::state::{Tag, SECONDS_IN_DAY},
    solana_account_decoder::UiAccountEncoding,
    solana_client::{
        rpc_client::RpcClient,
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    },
    solana_program::pubkey::Pubkey,
    solana_sdk::{account::Account, commitment_config::CommitmentConfig},
};
use access_protocol::state::StakePoolHeader;
use crate::settings::PROGRAM_ID;
use borsh::BorshDeserialize;

fn filter_pool_factory(contract_creation_timestamp: u64) -> impl Fn(&(Pubkey, StakePoolHeader)) -> bool {
    move |x| {
        let now = current_time();
        let (_, pool_header) = x;
        let pool_last_crank_time = pool_header.current_day_idx as u64 * SECONDS_IN_DAY + contract_creation_timestamp;
        now - pool_last_crank_time > SECONDS_IN_DAY
    }
}

fn deserialize_pool(x: &(Pubkey, Account)) -> (Pubkey, StakePoolHeader) {
    let (key, acc) = x;
    (*key, StakePoolHeader::deserialize(&mut &acc.data[..]).unwrap())
}

pub fn get_all_pools(connection: RpcClient, contract_creation_timestamp: u64) -> Result<Vec<Pubkey>, ProgramError> {

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
        .get_program_accounts_with_config(&PROGRAM_ID, config)
        .map_err(|_| ProgramError::Rpc)?;

    let keys: Vec<Pubkey> = result
        .iter()
        .map(deserialize_pool)
        .filter(filter_pool_factory(contract_creation_timestamp))
        .map(|(key, _)| key)
        .collect();

    Ok(keys)
}
