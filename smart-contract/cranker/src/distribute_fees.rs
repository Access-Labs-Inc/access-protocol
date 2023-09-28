use borsh::BorshDeserialize;
use solana_client::client_error::ClientError;
use solana_sdk::signature::Signature;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::solana_program::program_pack::Pack;

use {
    access_protocol::instruction::distribute_fees as distribute_fees_ix,
    solana_client::rpc_client::RpcClient,
    solana_program::pubkey::Pubkey,
    solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction},
};
use access_protocol::state::{CentralStateV2, MIN_DISTRIBUTE_AMOUNT};

use crate::{
    settings::{MINT, PAYER, PROGRAM_ID},
};

pub async fn distribute_fees(central_state: Pubkey) -> Result<(), Box<dyn std::error::Error>> {
    let connection = RpcClient::new(crate::settings::RPC_URL.as_str());
    let central_state_vault = get_associated_token_address(&central_state, &MINT);
    let central_state_vault_acc = spl_token::state::Account::unpack(&connection.get_account(&central_state_vault)?.data[..])?;
    println!("Central state vault balance: {}", central_state_vault_acc.amount);
    if central_state_vault_acc.amount < MIN_DISTRIBUTE_AMOUNT {
        println!("Central state vault balance too low, skipping fee distribution");
        return Ok(());
    }

    let recipient_atas = get_fee_recipient_atas(&connection, central_state).await?;

    let ix = distribute_fees_ix(
        *PROGRAM_ID,
        distribute_fees_ix::Accounts {
            fee_payer: &PAYER.pubkey(),
            central_state: &central_state,
            central_state_vault: &central_state_vault,
            spl_token_program: &spl_token::ID,
            mint: &MINT,
            token_accounts: recipient_atas.leak(),
        },
        access_protocol::instruction::distribute_fees::Params {},
    );

    let mut tx = Transaction::new_with_payer(&[ix], Some(&PAYER.pubkey()));


    let recent_blockhash = connection.get_latest_blockhash()?;
    tx.partial_sign::<Vec<&Keypair>>(&vec![&PAYER], recent_blockhash);
    let sig = connection.send_and_confirm_transaction(&tx)?;


    println!("Sent distribute fees tx - {:?}", sig);
    Ok(())
}

pub async fn get_fee_recipient_atas(connection: &RpcClient, central_state: Pubkey) -> Result<Vec<Pubkey>, Box<dyn std::error::Error>> {
    let acc = connection
        .get_account(&central_state)?;
    let central_state_stats = CentralStateV2::deserialize(&mut &acc.data[..])?;
    central_state_stats.recipients.iter().for_each(|r| {
        println!("{}: {}", r.owner, r.percentage);
    });

    let recipient_atas: Vec<Pubkey> = central_state_stats
        .recipients
        .iter()
        .map(|r| r.ata(&MINT))
        .collect();

    println!("Checking ATAs and creating the non-existent ones");
    central_state_stats
        .recipients
        .iter()
        .for_each(|r| { create_ata(connection, r.owner, &PAYER, &MINT)? })?;

    Ok(recipient_atas)
}

fn create_ata(
    connection: &RpcClient,
    owner: Pubkey,
    fee_payer: &Keypair,
    mint: &Pubkey,
) -> Result<Option<Signature>, ClientError> {
    let ata_key = get_associated_token_address(&owner, &MINT);
    println!("Checking ATA {}", ata_key);
    if let Ok(_) = connection.get_account(&ata_key) {
        println!("ATA {} exists", ata_key);
        return Ok(None)
    }
    println!("ATA {} doesn't exist, creating", ata_key);

    let ix = create_associated_token_account(
        &fee_payer.pubkey(),
        &owner,
        &mint,
        &spl_token::ID,
    );

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&fee_payer.pubkey()));
    let recent_blockhash = connection.get_latest_blockhash()?;
    transaction.sign(&vec![fee_payer], recent_blockhash);
    // not checking the result here as it will fail if the account already exists
    connection.send_and_confirm_transaction(&transaction).map(|sig| Some(sig))
}