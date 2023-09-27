use solana_program::clock::SECONDS_PER_DAY;
use solana_sdk::signer::Signer;

use access_protocol::state::Tag::BondAccount;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn v1_bonds() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    // ---------------------------------------------------------------------------------------------
    // Unlockable bond
    // ---------------------------------------------------------------------------------------------
    // Create users
    let pool_owner = tr.create_user_with_ata().await.unwrap();
    let bond_recipient = tr.create_user_with_ata().await.unwrap();
    // Create stake pool
    tr.create_pool(&pool_owner.pubkey(), 10_000)
        .await
        .unwrap();
    // Activate stake pool
    tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();

    let current_time = tr.get_current_time().await;
    let bond_amount = 20_000;

    // Create bond
    tr.create_bond(
        &pool_owner.pubkey(),
        &bond_recipient.pubkey(),
        bond_amount,
        1,
        5 * SECONDS_PER_DAY as i64,
        1,
    )
    .await
    .unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &bond_recipient.pubkey())
        .await
        .unwrap();

    let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.total_staked, bond_amount);
    assert_eq!(pool_stats.vault, bond_amount);
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.account.total_staked, bond_amount);
    let bond = tr
        .bond_stats(bond_recipient.pubkey(), pool_owner.pubkey(), bond_amount)
        .await
        .unwrap();
    assert_eq!(bond.tag, BondAccount);
    assert_eq!(
        bond.unlock_start_date,
        current_time + 5 * SECONDS_PER_DAY as i64
    );
    assert_eq!(bond.stake_pool, tr.get_pool_pda(&pool_owner.pubkey()));
    assert_eq!(bond.total_staked, bond_amount);
    assert_eq!(bond.owner, bond_recipient.pubkey());
    assert_eq!(bond.last_claimed_offset, 0);
    assert_eq!(bond.pool_minimum_at_creation, 10_000);

    // Claim zero rewards
    let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
    assert_eq!(recipient_stats.balance, 0);
    tr.claim_bond_rewards(&pool_owner.pubkey(), &bond_recipient)
        .await
        .unwrap_err();
    let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
    assert_eq!(recipient_stats.balance, 0);

    // Claim rewards
    _ = tr.sleep(SECONDS_PER_DAY).await;
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.claim_bond_rewards(&pool_owner.pubkey(), &bond_recipient)
        .await
        .unwrap();
    let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
    assert_eq!(recipient_stats.balance, 500_000);

    // Try unlocking - shouldn't be possible
    tr.unlock_bond(&pool_owner.pubkey(), &bond_recipient)
        .await
        .unwrap_err();

    // Move 5 days to the future
    _ = tr.sleep(5 * SECONDS_PER_DAY).await;
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    // Unlocking should not be possible before reward claim
    tr.unlock_bond(&pool_owner.pubkey(), &bond_recipient)
        .await
        .unwrap_err();
    _ = tr.sleep(1).await;
    // Claim rewards
    tr.claim_bond_rewards(&pool_owner.pubkey(), &bond_recipient)
        .await
        .unwrap();
    // Unlocking should be possible now
    tr.unlock_bond(&pool_owner.pubkey(), &bond_recipient)
        .await
        .unwrap();
    // Check all the stats
    let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
    assert_eq!(recipient_stats.balance, 2 * 500_000 + bond_amount);
    let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.total_staked, 0);
    assert_eq!(pool_stats.vault, 0);
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.account.total_staked, 0);
    let bond = tr
        .bond_stats(bond_recipient.pubkey(), pool_owner.pubkey(), bond_amount)
        .await
        .unwrap();
    assert_eq!(bond.tag, BondAccount);
    assert_eq!(
        bond.unlock_start_date,
        current_time + 5 * SECONDS_PER_DAY as i64
    );
    assert_eq!(bond.stake_pool, tr.get_pool_pda(&pool_owner.pubkey()));
    assert_eq!(bond.total_staked, 0);
    assert_eq!(bond.owner, bond_recipient.pubkey());
    assert_eq!(bond.last_claimed_offset, 6);
    assert_eq!(bond.pool_minimum_at_creation, 10_000);

    // Second unlock should not be possible
    // fixme but it is - what does it do?
    tr.unlock_bond(&pool_owner.pubkey(), &bond_recipient)
        .await
        .unwrap();
    let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
    assert_eq!(recipient_stats.balance, 2 * 500_000 + bond_amount);
    let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.total_staked, 0);
    assert_eq!(pool_stats.vault, 0);
    let central_state_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state_stats.account.total_staked, 0);
    let bond = tr
        .bond_stats(bond_recipient.pubkey(), pool_owner.pubkey(), bond_amount)
        .await
        .unwrap();
    assert_eq!(bond.tag, BondAccount);
    assert_eq!(
        bond.unlock_start_date,
        current_time + 5 * SECONDS_PER_DAY as i64
    );
    assert_eq!(bond.stake_pool, tr.get_pool_pda(&pool_owner.pubkey()));
    assert_eq!(bond.total_staked, 0);
    assert_eq!(bond.owner, bond_recipient.pubkey());
    assert_eq!(bond.last_claimed_offset, 6);
    assert_eq!(bond.pool_minimum_at_creation, 10_000);

    // Claim rewards in another 5 days
    _ = tr.sleep(5 * SECONDS_PER_DAY).await;
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.claim_bond_rewards(&pool_owner.pubkey(), &bond_recipient)
        .await
        .unwrap_err();
}
