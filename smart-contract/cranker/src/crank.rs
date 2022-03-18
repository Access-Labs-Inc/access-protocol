use {
    access_protocol::instruction::crank,
    solana_client::rpc_client::RpcClient,
    solana_program::pubkey::Pubkey,
    solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction},
};

use crate::{
    settings::PAYER,
    utils::{no_op_filter, retry},
};

pub async fn crank_pool(stake_pool: Pubkey, central_state: Pubkey) {
    let connection = RpcClient::new(crate::settings::RPC_URL.as_str());

    let ix = crank(
        access_protocol::ID,
        crank::Accounts {
            stake_pool: &stake_pool,
            central_state: &central_state,
        },
        crank::Params {},
    );

    let tx = Transaction::new_with_payer(&[ix], Some(&PAYER.pubkey()));

    let sig = retry(
        tx,
        |t| {
            let mut tr = t.clone();
            let recent_blockhash = connection.get_latest_blockhash()?;
            tr.partial_sign::<Vec<&Keypair>>(&vec![&PAYER], recent_blockhash);
            connection.send_and_confirm_transaction(&tr)
        },
        no_op_filter,
    )
    .await;

    println!("Sent crank tx for pool {} - {:?}", stake_pool, sig);
}
