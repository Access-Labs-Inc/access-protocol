use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use access_protocol::state::MAX_FEE_RECIPIENTS;
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

    // random MAX_FEE_RECIPIENTS recipients
    let mut recipients = vec![];
    for _ in 0..MAX_FEE_RECIPIENTS {
        recipients.push(tr.create_ata_account().await.unwrap());
    }
    let recipient_atas = recipients
        .iter()
        .map(|r| tr.get_ata(&r.pubkey()))
        .collect::<Vec<_>>();
    // MAX_FEE_RECIPIENTS - 1 random numbers between 1 and 5
    let recipient_percentages = (0..MAX_FEE_RECIPIENTS - 1)
        .map(|_| rand::random::<u64>() % 5 + 1)
        .collect::<Vec<_>>();
    // add one number so that the sum is 100
    let recipient_percentages = {
        let mut recipient_percentages = recipient_percentages;
        recipient_percentages.push(100 - recipient_percentages.iter().sum::<u64>());
        recipient_percentages
    };

    let staker = tr.create_ata_account().await.unwrap();
    tr.mint(&staker.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // todo test sum != 100

    // FeeRecipient { ata: recipient1_ata, percentage: 30 },
    let fee_recipients = recipients
        .iter()
        .zip(recipient_percentages.iter())
        .map(|(r, p)| FeeRecipient {
            owner: r.pubkey(),
            percentage: *p,
        })
        .collect::<Vec<_>>();

    tr.setup_fee_split(fee_recipients).await.unwrap();

    // this will add 10_000_000 to the fee split ata
    tr.stake(&pool_owner.pubkey(), &staker, 500_000_000)
        .await
        .unwrap();

    let fee_split_stats = tr.fee_split_stats().await.unwrap();
        assert_eq!(fee_split_stats.balance, 10_000_000);
    assert_eq!(fee_split_stats.recipients.len(), MAX_FEE_RECIPIENTS);


    tr.distribute_fees().await.unwrap();
    for (recipient, percentage) in recipients.iter().zip(recipient_percentages.iter()) {
        let recipient_stats = tr.staker_stats(recipient.pubkey()).await.unwrap();
        assert_eq!(recipient_stats.balance, 10_000_000 / 100 * percentage);
    }

    // check what happens when distributing 0 fees
    tr.sleep(1).await;
    tr.distribute_fees().await.unwrap();
    for (recipient, percentage) in recipients.iter().zip(recipient_percentages.iter()) {
        let recipient_stats = tr.staker_stats(recipient.pubkey()).await.unwrap();
        assert_eq!(recipient_stats.balance, 10_000_000 / 100 * percentage);
    }
}


