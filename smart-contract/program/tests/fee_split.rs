use solana_sdk::signer::Signer;

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

    let staker = tr.create_ata_account().await.unwrap();
    tr.mint(&staker.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    let fee_recipients = vec![
        FeeRecipient { address: recipient1.pubkey(), percentage: 30 },
        FeeRecipient { address: recipient2.pubkey(), percentage: 70 },
    ];

    tr.setup_fee_split(fee_recipients).await.unwrap();

    tr.stake(&pool_owner.pubkey(), &staker, 500_000_000)
        .await
        .unwrap();
}
