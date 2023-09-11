use solana_sdk::signer::Signer;

use access_protocol::state::FeeRecipient;
use access_protocol::state::{MAX_FEE_RECIPIENTS, MAX_FEE_SPLIT_SETUP_DELAY};

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
    // MAX_FEE_RECIPIENTS - 1 random numbers between 1 and 5
    let recipient_percentages = (0..MAX_FEE_RECIPIENTS - 1)
        .map(|_| rand::random::<u64>() % 5 + 1)
        .collect::<Vec<_>>();
    // add one number so that the sum is 99 (leaving 1% for the burn)
    let recipient_percentages = {
        let mut recipient_percentages = recipient_percentages;
        recipient_percentages.push(99 - recipient_percentages.iter().sum::<u64>());
        recipient_percentages
    };

    let staker = tr.create_ata_account().await.unwrap();
    tr.mint(&staker.pubkey(), 100_000_000_000).await.unwrap();
    tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    let fee_recipients = recipients
        .iter()
        .zip(recipient_percentages.iter())
        .map(|(r, p)| FeeRecipient {
            owner: r.pubkey(),
            percentage: *p,
        })
        .collect::<Vec<_>>();

    tr.setup_fee_split(fee_recipients).await.unwrap();

    // this adds 99_999_999 to the fee split ata
    tr.stake(&pool_owner.pubkey(), &staker, 4_999_999_949)
        .await
        .unwrap();

    let fee_split_stats = tr.fee_split_stats().await.unwrap();
    assert_eq!(fee_split_stats.balance, 99_999_999);
    assert_eq!(fee_split_stats.recipients.len(), MAX_FEE_RECIPIENTS);

    tr.distribute_fees().await.unwrap_err(); // not enough in account

    // this adds 1 to the fee split ata
    tr.stake(&pool_owner.pubkey(), &staker, 1).await.unwrap();
    let fee_split_stats = tr.fee_split_stats().await.unwrap();
    assert_eq!(fee_split_stats.balance, 100_000_000);
    assert_eq!(fee_split_stats.recipients.len(), MAX_FEE_RECIPIENTS);

    tr.sleep(1).await.unwrap();
    tr.distribute_fees().await.unwrap(); // not enough in account

    for (recipient, percentage) in recipients.iter().zip(recipient_percentages.iter()) {
        let recipient_stats = tr.staker_stats(recipient.pubkey()).await.unwrap();
        assert_eq!(recipient_stats.balance, 100_000_000 * percentage / 100);
    }

    // check what happens when distributing 0 fees
    tr.sleep(1).await.unwrap();
    tr.distribute_fees().await.unwrap_err();
    for (recipient, percentage) in recipients.iter().zip(recipient_percentages.iter()) {
        let recipient_stats = tr.staker_stats(recipient.pubkey()).await.unwrap();
        assert_eq!(recipient_stats.balance, 100_000_000 * percentage / 100);
    }

    // change the fee recipients
    let new_recipient1 = tr.create_ata_account().await.unwrap();
    let new_recipient2 = tr.create_ata_account().await.unwrap();

    tr.setup_fee_split(vec![
        FeeRecipient {
            owner: new_recipient1.pubkey(),
            percentage: 30,
        },
        FeeRecipient {
            owner: new_recipient2.pubkey(),
            percentage: 70,
        },
    ])
    .await
    .unwrap();

    // try changing with a wrong percentages
    tr.setup_fee_split(vec![
        FeeRecipient {
            owner: new_recipient1.pubkey(),
            percentage: 50,
        },
        FeeRecipient {
            owner: new_recipient2.pubkey(),
            percentage: 51,
        },
    ])
    .await
    .unwrap_err();

    // try changing with no recipients
    tr.setup_fee_split(vec![]).await.unwrap_err();

    // this will add 224691358 to the fee split ata
    tr.stake(&pool_owner.pubkey(), &staker, 11_234_567_891)
        .await
        .unwrap();

    let fee_split_stats = tr.fee_split_stats().await.unwrap();
    assert_eq!(fee_split_stats.balance, 224691358);

    // try changing the recipients too late after the last distribution
    tr.sleep(MAX_FEE_SPLIT_SETUP_DELAY + 1).await.unwrap();
    tr.setup_fee_split(vec![FeeRecipient {
        owner: new_recipient1.pubkey(),
        percentage: 3,
    }])
    .await
    .unwrap_err();

    tr.distribute_fees().await.unwrap();

    let recipient1_stats = tr.staker_stats(new_recipient1.pubkey()).await.unwrap();
    assert_eq!(recipient1_stats.balance, 224691358 * 30 / 100);
    let recipient2_stats = tr.staker_stats(new_recipient2.pubkey()).await.unwrap();
    assert_eq!(recipient2_stats.balance, 224691358 * 70 / 100);
    let fee_split_stats = tr.fee_split_stats().await.unwrap();
    assert_eq!(fee_split_stats.balance, 0);

    // now the change should be possible
    tr.sleep(1).await.unwrap();
    tr.setup_fee_split(vec![FeeRecipient {
        owner: new_recipient1.pubkey(),
        percentage: 3,
    }])
    .await
    .unwrap();
}
