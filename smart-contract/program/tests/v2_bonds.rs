use solana_sdk::signer::Signer;
use access_protocol::state::FEES;
use access_protocol::state::Tag::BondAccountV2;
use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn signed_claim() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    // Create users
    let stake_pool_owner = tr.create_ata_account().await.unwrap();
    let from = tr.create_ata_account().await.unwrap();
    let to = tr.create_ata_account().await.unwrap();
    // Mint to staker
    tr.mint(&from.pubkey(), 100_000).await.unwrap();
    // Create stake pool
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 10_000).await.unwrap();
    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey()).await.unwrap();
    // Create real bond with quote amount
    let current_time = tr.get_current_time().await;
    let unlock_date = current_time + 1000;
    let bond_amount = 20_000;
    tr.create_bond_v2(
        &from,
        &to.pubkey(),
        &stake_pool_owner.pubkey(),
        bond_amount,
        Some(unlock_date),
    ).await.unwrap();

        let staker_stats = tr.staker_stats(from.pubkey()).await.unwrap();
        assert_eq!(staker_stats.balance, 100_000 - bond_amount - bond_amount * FEES / 100);
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.total_pool_staked, bond_amount);
        let central_state_stats = tr.central_state_stats().await.unwrap();
        assert_eq!(central_state_stats.total_staked, bond_amount);
    let bond = tr.bond_v2_stats(to.pubkey(), stake_pool_owner.pubkey(),  Some(unlock_date)).await.unwrap();
    assert_eq!(bond.tag, BondAccountV2);
    assert_eq!(bond.unlock_date, Some(unlock_date));
    assert_eq!(bond.pool, tr.get_pool_pda(&stake_pool_owner.pubkey()));
    assert_eq!(bond.amount, bond_amount);
    assert_eq!(bond.owner, to.pubkey());
    assert_eq!(bond.last_claimed_offset, 0);
    assert_eq!(bond.pool_minimum_at_creation, 10_000);

    // Wait 5 minutes
    // tr.sleep(300).await.unwrap();
    // // Create real bond with quote amount
    // tr.create_bond_v2(
    //     &from,
    //     &to.pubkey(),
    //     &stake_pool_owner.pubkey(),
    //     10000,
    //     Some(unlock_date),
    // ).await.unwrap();
    // let staker_stats = tr.staker_stats(from.pubkey()).await.unwrap();
    // assert_eq!(staker_stats.balance, 100_000 - 10_000 - 200 - 10_000 - 200);
    // let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    // assert_eq!(pool_stats.total_pool_staked, 10_000 + 10_000);
    // let central_state_stats = tr.central_state_stats().await.unwrap();
    // assert_eq!(central_state_stats.total_staked, 10_000 + 10_000);

}