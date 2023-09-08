use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;

use access_protocol::state::FeeRecipient;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn fee_split() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();

    let pool_owner = tr.create_ata_account().await.unwrap();
    tr.create_stake_pool(&pool_owner.pubkey(), 200_000_000)
        .await
        .unwrap();
    tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();

    let recipient1 = tr.create_ata_account().await.unwrap();
    let recipient2 = tr.create_ata_account().await.unwrap();
    let recipient1_ata = tr.get_ata(&recipient1.pubkey());
        let recipient2_ata = tr.get_ata(&recipient2.pubkey());

    let staker = tr.create_ata_account().await.unwrap();
    tr.mint(&staker.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    let fee_recipients = vec![
        FeeRecipient { ata: recipient1_ata, percentage: 30 },
        FeeRecipient { ata: recipient2_ata, percentage: 70 },
    ];

    tr.setup_fee_split(fee_recipients).await.unwrap();

    // this will add 10_000_000 to the fee split ata
    tr.stake(&pool_owner.pubkey(), &staker, 500_000_000)
        .await
        .unwrap();

    let fee_split_stats = tr.fee_split_stats().await.unwrap();
        assert_eq!(fee_split_stats.balance, 10_000_000);
    assert_eq!(fee_split_stats.recipients.len(), 2);


    tr.distribute_fees().await.unwrap();
    let recipient1_stats = tr.staker_stats(recipient1.pubkey()).await.unwrap();
    assert_eq!(recipient1_stats.balance, 3_000_000);
    let recipient2_stats = tr.staker_stats(recipient2.pubkey()).await.unwrap();
    assert_eq!(recipient2_stats.balance, 7_000_000);

    // todo check what happens when distributing 0 fees
}


