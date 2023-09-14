


use solana_sdk::signature::Signer;

use access_protocol::state::FeeRecipient;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn change_protocol_fee() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    let staker = tr.create_ata_account().await.unwrap();
    tr.setup_fee_split(vec![FeeRecipient {
        owner: staker.pubkey(),
        percentage: 100,
    }]).await.unwrap();

    let pool_owner = tr.create_ata_account().await.unwrap();
    tr.create_stake_pool(&pool_owner.pubkey(), 10_000_000)
        .await
        .unwrap();
    tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();

    let staker = tr.create_ata_account().await.unwrap();
    tr.mint(&staker.pubkey(), 100_000_000).await.unwrap();
    tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // Check the fee basis points
    let stats = tr.fee_split_stats().await.unwrap();
    assert_eq!(stats.fee_basis_points, 200);
    tr.stake(&pool_owner.pubkey(), &staker, 10_000_000)
        .await
        .unwrap();

        // Set the fee basis points - should fail as it is over 100%
        tr.change_protocol_fee(10_001).await.unwrap_err();
    let stats = tr.fee_split_stats().await.unwrap();
    assert_eq!(stats.fee_basis_points, 200);
    assert_eq!(stats.balance, 200_000);

    // Set the fee basis points - should succeed
    tr.sleep(1).await.unwrap();
    tr.change_protocol_fee(3_000).await.unwrap();

    // Check the fee basis points
    tr.stake(&pool_owner.pubkey(), &staker, 10_000_000)
        .await
        .unwrap();
    let stats = tr.fee_split_stats().await.unwrap();
    assert_eq!(stats.fee_basis_points, 3_000);
    assert_eq!(stats.balance, 3_200_000);
}