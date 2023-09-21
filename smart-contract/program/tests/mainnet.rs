use std::str::FromStr;

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;

use access_protocol::state::StakeAccount;

pub mod common;

#[test]
fn mainnet() {
    let rpc_url = ""; // todo fill
    let rpc_client = RpcClient::new(rpc_url.to_owned());

    let program_id = Pubkey::from_str("6HW8dXjtiTGkD4jzXs7igdFmZExPpmwUrRN5195xGup").unwrap();

    let (stake_acc_key, _stake_nonce) = Pubkey::find_program_address(
        &[
            "stake_account".as_bytes(),
            &Pubkey::from_str("BySCc6DnNEeparG8kYHiWHQ4yi2rAaxJHdmYsJ3r8vXU")
                .unwrap()
                .to_bytes(),
            &Pubkey::from_str("Fxh4hDFHJuTfD3Eq4en36dTk8QvbsSMoTE5Y2hVX3qVt")
                .unwrap()
                .to_bytes(),
        ],
        &program_id,
    );
    println!("stake_acc_key: {:?}", stake_acc_key);

    let acc = rpc_client.get_account(&stake_acc_key).unwrap();

    let account = StakeAccount::deserialize(&mut &acc.data[..]).unwrap();
    println!("account: {:?}", account);
}
