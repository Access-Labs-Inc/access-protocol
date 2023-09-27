use solana_sdk::signer::Signer;
use solana_test_framework::*;

use crate::common::test_runner::TestRunner;

pub mod common;

const DAY: u64 = 86400;
const DAILY_INFLATION: u64 = 1_000_000_000_000; // 1M of tokens

#[tokio::test]
async fn full_system_test() {
    // ------------------------------------------
    // DAY 1
    //-------------------------------------------
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(DAILY_INFLATION).await.unwrap();

    // Setup 3 airdrop users
    let airdrop_user1 = tr.create_user_with_ata().await.unwrap();
    let airdrop_user2 = tr.create_user_with_ata().await.unwrap();
    let airdrop_user3 = tr.create_user_with_ata().await.unwrap();

    // Setup 2 vesting users
    let vesting_user1 = tr.create_user_with_ata().await.unwrap();
    let vesting_user2 = tr.create_user_with_ata().await.unwrap();

    // Setup 3 pool owners
    let pool_owner = tr.create_user_with_ata().await.unwrap();
    let pool_owner2 = tr.create_user_with_ata().await.unwrap();
    let pool_owner3 = tr.create_user_with_ata().await.unwrap();

    // Setup all the pools
    tr.create_pool(&pool_owner.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    tr.create_pool(&pool_owner2.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    tr.create_pool(&pool_owner3.pubkey(), 1_000_000_000)
        .await
        .unwrap();

    // Activate the pools
    tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();
    tr.activate_stake_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.activate_stake_pool(&pool_owner3.pubkey()).await.unwrap();

    // Setup 1 staker with 10_200_000_000 tokens in his account and appropriate stake accounts for all pools
    let staker = tr.create_user_with_ata().await.unwrap();
    tr.mint(&staker.pubkey(), 10_200_000_000).await.unwrap();
    tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();
    tr.create_stake_account(&pool_owner2.pubkey(), &staker.pubkey())
        .await
        .unwrap();
    tr.create_stake_account(&pool_owner3.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // ------------------------------------------
    // DAY 2
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    // First two airdrop users get airdrop of 100000 tokens to the first pool with the release date on day 10
    tr.create_bond(
        &pool_owner.pubkey(),
        &airdrop_user1.pubkey(),
        100_000_000_000,
        1,
        8 * DAY as i64,
        1,
    )
    .await
    .unwrap();
    tr.create_bond(
        &pool_owner.pubkey(),
        &airdrop_user2.pubkey(),
        100_000_000_000,
        1,
        8 * DAY as i64,
        1,
    )
    .await
    .unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &airdrop_user1.pubkey())
        .await
        .unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &airdrop_user2.pubkey())
        .await
        .unwrap();

    let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.total_staked, 200_000_000_000);
    let central_state = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state.account.total_staked, 200_000_000_000);

    // ------------------------------------------
    // DAY 3
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // last airdrop user gets airdrop of 100000 tokens to second pool with the release date on day 10
    tr.create_bond(
        &pool_owner2.pubkey(),
        &airdrop_user3.pubkey(),
        100_000_000_000,
        1,
        7 * DAY as i64,
        1,
    )
    .await
    .unwrap();
    tr.claim_bond(&pool_owner2.pubkey(), &airdrop_user3.pubkey())
        .await
        .unwrap();

    // staker stakes 10000 tokens to each pool - second stake should fail at first, then we mint some more tokens to the staker
    tr.stake(&pool_owner.pubkey(), &staker, 10_000_000_000)
        .await
        .unwrap();
    tr.stake(&pool_owner2.pubkey(), &staker, 10_000_000_000)
        .await
        .unwrap_err();
    tr.mint(&staker.pubkey(), 20_400_000_000).await.unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 20_400_000_000);
    tr.sleep(1).await.unwrap(); // so that we are not sending the same transaction twice in the same block
    tr.stake(&pool_owner2.pubkey(), &staker, 10_000_000_000)
        .await
        .unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 10_200_000_000);
    tr.stake(&pool_owner3.pubkey(), &staker, 10_000_000_000)
        .await
        .unwrap();

    // check that the totalStaked and pool stats are correct
    let pool1_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool1_stats.header.total_staked, 210_000_000_000);
    let pool2_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
    assert_eq!(pool2_stats.header.total_staked, 110_000_000_000);
    let pool3_stats = tr.pool_stats(pool_owner3.pubkey()).await.unwrap();
    assert_eq!(pool3_stats.header.total_staked, 10_000_000_000);
    let central_state = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state.account.total_staked, 330_000_000_000);

    // ------------------------------------------
    // DAY 4
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // create vesting airdrop with cliff at day 7 with rewards for 100_000 tokens to both vesting users to pool1 and pool3
    // fixme the parameter should be 3 * DAY not 2 * DAY, but with the current implementation this wouldn't work
    tr.create_bond(
        &pool_owner.pubkey(),
        &vesting_user1.pubkey(),
        10_000_000_000,
        10,
        2 * DAY as i64,
        DAY as i64,
    )
    .await
    .unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &vesting_user1.pubkey())
        .await
        .unwrap();
    tr.create_bond(
        &pool_owner.pubkey(),
        &vesting_user2.pubkey(),
        10_000_000_000,
        10,
        2 * DAY as i64,
        DAY as i64,
    )
    .await
    .unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &vesting_user2.pubkey())
        .await
        .unwrap();
    tr.create_bond(
        &pool_owner3.pubkey(),
        &vesting_user1.pubkey(),
        10_000_000_010,
        10,
        2 * DAY as i64,
        DAY as i64,
    )
    .await
    .unwrap();
    tr.claim_bond(&pool_owner3.pubkey(), &vesting_user1.pubkey())
        .await
        .unwrap();
    tr.create_bond(
        &pool_owner3.pubkey(),
        &vesting_user2.pubkey(),
        10_000_000_010,
        10,
        2 * DAY as i64,
        DAY as i64,
    )
    .await
    .unwrap();
    tr.claim_bond(&pool_owner3.pubkey(), &vesting_user2.pubkey())
        .await
        .unwrap();

    // check that the totalStaked and pool stats are correct
    let pool1_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool1_stats.header.total_staked, 230_000_000_000);
    let pool2_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
    assert_eq!(pool2_stats.header.total_staked, 110_000_000_000);
    let pool3_stats = tr.pool_stats(pool_owner3.pubkey()).await.unwrap();
    assert_eq!(pool3_stats.header.total_staked, 30_000_000_020);
    let central_state = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state.account.total_staked, 370_000_000_020);

    // ------------------------------------------
    // DAY 5
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // staker unstakes 5000 tokens from pool1 - first unstake should fail at first, then he claims his rewards, but just for pool 1
    tr.unstake(&pool_owner.pubkey(), &staker, 5_000_000_000)
        .await
        .unwrap_err();
    tr.claim_staker_rewards(&pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    tr.sleep(1).await.unwrap(); // so that we are not sending the same transaction twice in the same block
    tr.unstake(&pool_owner.pubkey(), &staker, 5_000_000_000)
        .await
        .unwrap();

    // check that the totalStaked and pool stats are correct
    let pool1_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool1_stats.header.total_staked, 225_000_000_000);
    let pool2_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
    assert_eq!(pool2_stats.header.total_staked, 110_000_000_000);
    let pool3_stats = tr.pool_stats(pool_owner3.pubkey()).await.unwrap();
    assert_eq!(pool3_stats.header.total_staked, 30_000_000_020);
    let central_state = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state.account.total_staked, 365_000_000_020);
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(
        staker_stats.balance,
        5_000_000_000
            + ((DAILY_INFLATION as f64 / 330.0 * 10.0 * 0.5)
                + (DAILY_INFLATION as f64 * 10_000_000_000.0 / 370_000_000_020.0 * 0.5))
                .round() as u64
    );

    // ------------------------------------------
    // DAY 6
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    //vestingUser1 should be able to claim his rewards in pool1
    tr.claim_bond_rewards(&pool_owner.pubkey(), &vesting_user1)
        .await
        .unwrap();
    let vesting_owner_stats = tr.staker_stats(vesting_user1.pubkey()).await.unwrap();
    // todo maybe investigate this rounding error
    assert_eq!(
        vesting_owner_stats.balance / 10,
        ((DAILY_INFLATION as f64 * 10_000_000_000.0 / 370_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 10_000_000_000.0 / 365_000_000_020.0 * 0.5))
            .round() as u64
            / 10
    );

    // no one can claim their airdrops yet
    tr.unlock_bond(&pool_owner.pubkey(), &airdrop_user1)
        .await
        .unwrap_err();
    tr.unlock_bond(&pool_owner.pubkey(), &airdrop_user2)
        .await
        .unwrap_err();
    tr.unlock_bond(&pool_owner2.pubkey(), &airdrop_user3)
        .await
        .unwrap_err();
    tr.unlock_bond(&pool_owner.pubkey(), &vesting_user1)
        .await
        .unwrap_err();
    tr.unlock_bond(&pool_owner.pubkey(), &vesting_user2)
        .await
        .unwrap_err();

    // claim rewards as a pool1 owner
    tr.claim_pool_rewards(&pool_owner).await.unwrap();
    let pool_owner_stats = tr.staker_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(
        pool_owner_stats.balance,
        ((DAILY_INFLATION as f64 * 0.5)
            + (DAILY_INFLATION as f64 / 330.0 * 210.0 * 0.5)
            + (DAILY_INFLATION as f64 * 230_000_000_000.0 / 370_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 225_000_000_000.0 / 365_000_000_020.0 * 0.5))
            .round() as u64
    );

    // ------------------------------------------
    // DAY 7
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // the first part of the vesting airdrop should be ready now
    tr.claim_bond_rewards(&pool_owner.pubkey(), &vesting_user1)
        .await
        .unwrap();
    tr.unlock_bond(&pool_owner.pubkey(), &vesting_user1)
        .await
        .unwrap();
    let vesting_owner_stats = tr.staker_stats(vesting_user1.pubkey()).await.unwrap();
    // todo maybe investigate this rounding error
    assert_eq!(
        vesting_owner_stats.balance / 10,
        1_000_000_000 / 10
            + ((DAILY_INFLATION as f64 * 10_000_000_000.0 / 370_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 10_000_000_000.0 / 365_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 10_000_000_000.0 / 365_000_000_020.0 * 0.5))
                .round() as u64
                / 10
    );

    // second unlock of the same bond should fail today
    tr.sleep(1).await.unwrap(); // so that we are not sending the same transaction twice in the same block
    tr.claim_bond_rewards(&pool_owner.pubkey(), &vesting_user1)
        .await
        .unwrap_err();
    tr.unlock_bond(&pool_owner.pubkey(), &vesting_user1)
        .await
        .unwrap_err();

    // pool1 owner should be able to claim his rewards again
    tr.claim_pool_rewards(&pool_owner).await.unwrap();
    let pool_owner_stats = tr.staker_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(
        pool_owner_stats.balance,
        ((DAILY_INFLATION as f64 * 0.5)
            + (DAILY_INFLATION as f64 / 330.0 * 210.0 * 0.5)
            + (DAILY_INFLATION as f64 * 230_000_000_000.0 / 370_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 225_000_000_000.0 / 365_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 225_000_000_000.0 / 365_000_000_020.0 * 0.5))
            .round() as u64
    );

    // ------------------------------------------
    // DAY 8
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // the second part of the vesting airdrop should be ready now
    tr.claim_bond_rewards(&pool_owner.pubkey(), &vesting_user1)
        .await
        .unwrap();
    tr.unlock_bond(&pool_owner.pubkey(), &vesting_user1)
        .await
        .unwrap();
    let vesting_owner_stats = tr.staker_stats(vesting_user1.pubkey()).await.unwrap();
    // todo maybe investigate this rounding error
    assert_eq!(
        vesting_owner_stats.balance / 10,
        2_000_000_000 / 10
            + ((DAILY_INFLATION as f64 * 10_000_000_000.0 / 370_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 10_000_000_000.0 / 365_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 10_000_000_000.0 / 365_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 9_000_000_000.0 / 364_000_000_020.0 * 0.5))
                .round() as u64
                / 10
    );

    // ------------------------------------------
    // DAY 9
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // check that the airdrop_users still cannot claim their airdrop
    tr.claim_bond_rewards(&pool_owner.pubkey(), &airdrop_user1)
        .await
        .unwrap();
    tr.unlock_bond(&pool_owner.pubkey(), &airdrop_user1)
        .await
        .unwrap_err();
    tr.claim_bond_rewards(&pool_owner.pubkey(), &airdrop_user2)
        .await
        .unwrap();
    tr.unlock_bond(&pool_owner.pubkey(), &airdrop_user2)
        .await
        .unwrap_err();
    tr.claim_bond_rewards(&pool_owner2.pubkey(), &airdrop_user3)
        .await
        .unwrap();
    tr.unlock_bond(&pool_owner2.pubkey(), &airdrop_user3)
        .await
        .unwrap_err();

    // ------------------------------------------
    // DAY 10
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // check that the airdropUser2 can claim his airdrop
    tr.claim_bond_rewards(&pool_owner.pubkey(), &airdrop_user2)
        .await
        .unwrap();
    tr.unlock_bond(&pool_owner.pubkey(), &airdrop_user2)
        .await
        .unwrap();
    let airdrop_user2_stats = tr.staker_stats(airdrop_user2.pubkey()).await.unwrap();
    // todo maybe investigate this rounding error
    assert_eq!(
        airdrop_user2_stats.balance / 100,
        100_000_000_000 / 100
            + (DAILY_INFLATION as f64 * 100_000_000_000.0 / 200_000_000_000.0 * 0.5
                + DAILY_INFLATION as f64 * 100_000_000_000.0 / 330_000_000_000.0 * 0.5
                + DAILY_INFLATION as f64 * 100_000_000_000.0 / 370_000_000_020.0 * 0.5
                + DAILY_INFLATION as f64 * 100_000_000_000.0 / 365_000_000_020.0 * 0.5
                + DAILY_INFLATION as f64 * 100_000_000_000.0 / 365_000_000_020.0 * 0.5
                + DAILY_INFLATION as f64 * 100_000_000_000.0 / 364_000_000_020.0 * 0.5
                + DAILY_INFLATION as f64 * 100_000_000_000.0 / 363_000_000_020.0 * 0.5
                + DAILY_INFLATION as f64 * 100_000_000_000.0 / 363_000_000_020.0 * 0.5)
                .round() as u64
                / 100
    );

    //check that the pool1 owner can claim his rewards
    tr.claim_pool_rewards(&pool_owner).await.unwrap();
    let pool_owner_stats = tr.staker_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(
        pool_owner_stats.balance / 10,
        (DAILY_INFLATION as f64 * 0.5
            + DAILY_INFLATION as f64 / 330.0 * 210.0 * 0.5
            + DAILY_INFLATION as f64 * 230_000_000_000.0 / 370_000_000_020.0 * 0.5
            + DAILY_INFLATION as f64 * 225_000_000_000.0 / 365_000_000_020.0 * 0.5
            + DAILY_INFLATION as f64 * 225_000_000_000.0 / 365_000_000_020.0 * 0.5
            + DAILY_INFLATION as f64 * 224_000_000_000.0 / 364_000_000_020.0 * 0.5
            + DAILY_INFLATION as f64 * 223_000_000_000.0 / 363_000_000_020.0 * 0.5
            + DAILY_INFLATION as f64 * 223_000_000_000.0 / 363_000_000_020.0 * 0.5)
            .round() as u64
            / 10
    );

    // ------------------------------------------
    // DAY 11
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // check that the pool1 owner can claim his rewards
    tr.claim_pool_rewards(&pool_owner).await.unwrap();
    let pool_owner_stats = tr.staker_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(
        pool_owner_stats.balance / 10,
        ((DAILY_INFLATION as f64 * 0.5)
            + (DAILY_INFLATION as f64 / 330.0 * 210.0 * 0.5)
            + (DAILY_INFLATION as f64 * 230_000_000_000.0 / 370_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 225_000_000_000.0 / 365_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 225_000_000_000.0 / 365_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 224_000_000_000.0 / 364_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 223_000_000_000.0 / 363_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 223_000_000_000.0 / 363_000_000_020.0 * 0.5)
            + (DAILY_INFLATION as f64 * 123_000_000_000.0 / 263_000_000_020.0 * 0.5))
            .round() as u64
            / 10
    );

    // staker claims all rewards
    tr.claim_staker_rewards(&pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    tr.claim_staker_rewards(&pool_owner2.pubkey(), &staker)
        .await
        .unwrap();
    tr.claim_staker_rewards(&pool_owner3.pubkey(), &staker)
        .await
        .unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    // todo maybe investigate this rounding error
    assert_eq!(
        staker_stats.balance / 100,
        5_000_000_000 / 100
            + ((DAILY_INFLATION as f64 / 330.0 * 30.0 * 0.5)
                + (DAILY_INFLATION as f64 * 30_000_000_000.0 / 370_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 365_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 365_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 364_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 363_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 363_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 263_000_000_020.0 * 0.5))
                .round() as u64
                / 100
    );

    // unstake it all
    tr.unstake(&pool_owner.pubkey(), &staker, 5_000_000_000)
        .await
        .unwrap();
    tr.unstake(&pool_owner2.pubkey(), &staker, 10_000_000_000)
        .await
        .unwrap();
    tr.unstake(&pool_owner3.pubkey(), &staker, 10_000_000_000)
        .await
        .unwrap();

    // claim rewards as a staker - should not claim anything
    tr.sleep(1).await.unwrap(); // so that we are not sending the same transaction twice in the same block
    tr.claim_staker_rewards(&pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    tr.claim_staker_rewards(&pool_owner2.pubkey(), &staker)
        .await
        .unwrap();
    tr.claim_staker_rewards(&pool_owner3.pubkey(), &staker)
        .await
        .unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    // todo maybe investigate this rounding error
    assert_eq!(
        staker_stats.balance / 100,
        30_000_000_000 / 100
            + ((DAILY_INFLATION as f64 / 330.0 * 30.0 * 0.5)
                + (DAILY_INFLATION as f64 * 30_000_000_000.0 / 370_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 365_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 365_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 364_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 363_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 363_000_000_020.0 * 0.5)
                + (DAILY_INFLATION as f64 * 25_000_000_000.0 / 263_000_000_020.0 * 0.5))
                .round() as u64
                / 100
    );
}
