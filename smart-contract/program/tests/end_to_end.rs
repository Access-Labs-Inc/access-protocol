use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;

use solana_test_framework::*;

use access_protocol::state::Tag;

use crate::common::test_runner::TestRunner;

pub mod common;

const DAILY_INFLATION: u64 = 1_000_000;

#[tokio::test]
async fn end_to_end() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(DAILY_INFLATION).await.unwrap();

    let cs_stats = tr.central_state_stats().await.unwrap();
    assert_eq!(cs_stats.tag, Tag::CentralState);
    assert_eq!(cs_stats.daily_inflation, DAILY_INFLATION);
    assert_eq!(cs_stats.token_mint.to_string(), tr.get_mint().to_string());
    assert_eq!(cs_stats.authority, tr.get_authority());
    assert_eq!(cs_stats.creation_time, tr.get_current_time().await);
    assert_eq!(cs_stats.total_staked, 0);
    assert_eq!(cs_stats.total_staked_snapshot, 0);
    assert_eq!(cs_stats.last_snapshot_offset, 0);

    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    let staker = tr.create_user_with_ata().await.unwrap();

    // Create stake pool
    tr.create_stake_pool(&stake_pool_owner.pubkey(), 1000)
        .await
        .unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.tag, Tag::InactiveStakePool as u8);
    assert_eq!(pool_stats.header.current_day_idx, 0);
    assert_eq!(pool_stats.header.minimum_stake_amount, 1000);
    assert_eq!(pool_stats.header.total_staked, 0);
    assert_eq!(pool_stats.header.last_claimed_offset, 0);
    assert_eq!(pool_stats.header.stakers_part, 50);
    assert_eq!(
        Pubkey::new(&pool_stats.header.owner).to_string(),
        stake_pool_owner.pubkey().to_string()
    );

    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();
    let pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.header.tag, Tag::StakePool as u8);

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();
    let stake_account_stats = tr
        .stake_account_stats(staker.pubkey(), stake_pool_owner.pubkey())
        .await
        .unwrap();
    assert_eq!(stake_account_stats.tag, Tag::StakeAccount);
    assert_eq!(stake_account_stats.owner, staker.pubkey());
    assert_eq!(stake_account_stats.stake_amount, 0);
    let stake_pool_pda_key = tr.get_pool_pda(&stake_pool_owner.pubkey());
    assert_eq!(stake_account_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(stake_account_stats.last_claimed_offset, 0);
    assert_eq!(stake_account_stats.pool_minimum_at_creation, 1000);

    // Create a bond
    let bond_amount = 5_000_000;
    let unlock_after = 10;
    tr.create_bond(
        &stake_pool_owner.pubkey(),
        &staker.pubkey(),
        bond_amount,
        1,
        unlock_after,
        1,
    )
    .await
    .unwrap();
    let bond_stats = tr
        .bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount)
        .await
        .unwrap();
    assert_eq!(bond_stats.tag, Tag::InactiveBondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, bond_amount);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(
        bond_stats.unlock_start_date,
        tr.get_current_time().await + unlock_after
    );
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    assert_eq!(
        bond_stats.last_unlock_time,
        tr.get_current_time().await + unlock_after
    );
    assert_eq!(bond_stats.total_unlocked_amount, 0);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 0);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());

    // Claim bond
    tr.claim_bond(&stake_pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();
    let bond_stats = tr
        .bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount)
        .await
        .unwrap();
    let bond_creation_time = tr.get_current_time().await;
    assert_eq!(bond_stats.tag, Tag::BondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, bond_amount);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(bond_stats.quote_mint, tr.get_mint());
    assert_eq!(bond_stats.seller_token_account, tr.get_bond_seller_ata());
    assert_eq!(
        bond_stats.unlock_start_date,
        bond_creation_time + unlock_after
    );
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    assert_eq!(
        bond_stats.last_unlock_time,
        bond_creation_time + unlock_after
    );
    assert_eq!(bond_stats.total_unlocked_amount, 0);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 0);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());

    let _stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();

    // Crank
    let sleep_time = (86_400 * 3 / 2) as u64;
    tr.sleep(sleep_time).await.unwrap();
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Unlock bond should fail - unclaimed rewards
    tr.unlock_bond(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap_err();
    tr.sleep(1).await.unwrap();

    // Claim bond rewards
    tr.claim_bond_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let bond_stats = tr
        .bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount)
        .await
        .unwrap();
    assert_eq!(bond_stats.tag, Tag::BondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, bond_amount);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(bond_stats.quote_mint, tr.get_mint());
    assert_eq!(bond_stats.seller_token_account, tr.get_bond_seller_ata());
    assert_eq!(
        bond_stats.unlock_start_date,
        bond_creation_time + unlock_after
    );
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    // assert_eq!(bond_stats.last_unlock_time, bond_creation_time + unlock_after);
    assert_eq!(bond_stats.total_unlocked_amount, 0);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 1);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, DAILY_INFLATION / 2);

    // Unlock bond
    tr.unlock_bond(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let bond_stats = tr
        .bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount)
        .await
        .unwrap();
    assert_eq!(bond_stats.tag, Tag::BondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, 0);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(bond_stats.quote_mint, tr.get_mint());
    assert_eq!(bond_stats.seller_token_account, tr.get_bond_seller_ata());
    assert_eq!(
        bond_stats.unlock_start_date,
        bond_creation_time + unlock_after
    );
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    assert_eq!(bond_stats.last_unlock_time, tr.get_current_time().await);
    assert_eq!(bond_stats.total_unlocked_amount, bond_amount);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 1);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());

    // Stake
    let stake_amount = 10_000;
    tr.stake(&stake_pool_owner.pubkey(), &staker, stake_amount)
        .await
        .unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 4989800 + DAILY_INFLATION / 2);
    let stake_account_stats = tr
        .stake_account_stats(staker.pubkey(), stake_pool_owner.pubkey())
        .await
        .unwrap();
    assert_eq!(stake_account_stats.tag, Tag::StakeAccount);
    assert_eq!(stake_account_stats.owner, staker.pubkey());
    assert_eq!(stake_account_stats.stake_amount, stake_amount);
    assert_eq!(stake_account_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(stake_account_stats.last_claimed_offset, 1);
    assert_eq!(stake_account_stats.pool_minimum_at_creation, 1000);

    let _stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();

    // Crank
    tr.sleep(86_400 / 2 + 1000).await.unwrap();
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();
    let stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_pool_stats.header.tag, Tag::StakePool as u8);
    assert_eq!(stake_pool_stats.header.current_day_idx, 2);
    assert_eq!(stake_pool_stats.header.minimum_stake_amount, 1000);
    assert_eq!(stake_pool_stats.header.total_staked, stake_amount);
    assert_eq!(stake_pool_stats.header.last_claimed_offset, 0);
    assert_eq!(stake_pool_stats.header.stakers_part, 50);
    assert_eq!(
        Pubkey::new(&pool_stats.header.owner).to_string(),
        stake_pool_owner.pubkey().to_string()
    );

    // Claim bond rewards
    tr.claim_bond_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let bond_stats = tr
        .bond_stats(staker.pubkey(), stake_pool_owner.pubkey(), bond_amount)
        .await
        .unwrap();
    assert_eq!(bond_stats.tag, Tag::BondAccount);
    assert_eq!(bond_stats.owner, staker.pubkey());
    assert_eq!(bond_stats.total_amount_sold, bond_amount);
    assert_eq!(bond_stats.total_staked, 0);
    assert_eq!(bond_stats.total_quote_amount, 0);
    assert_eq!(bond_stats.quote_mint, tr.get_mint());
    assert_eq!(bond_stats.seller_token_account, tr.get_bond_seller_ata());
    assert_eq!(
        bond_stats.unlock_start_date,
        bond_creation_time + unlock_after
    );
    assert_eq!(bond_stats.unlock_period, 1);
    assert_eq!(bond_stats.unlock_amount, bond_amount);
    // assert_eq!(bond_stats.last_unlock_time, bond_creation_time + unlock_after);
    assert_eq!(bond_stats.total_unlocked_amount, bond_amount);
    assert_eq!(bond_stats.pool_minimum_at_creation, 1000);
    assert_eq!(bond_stats.stake_pool, stake_pool_pda_key);
    assert_eq!(bond_stats.last_claimed_offset, 2);
    assert_eq!(bond_stats.sellers[0], tr.get_bond_seller());
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 4989800 + DAILY_INFLATION / 2);

    // Claim pool rewards
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_pool_stats.header.tag, Tag::StakePool as u8);
    assert_eq!(stake_pool_stats.header.current_day_idx, 2);
    assert_eq!(stake_pool_stats.header.minimum_stake_amount, 1000);
    assert_eq!(stake_pool_stats.header.total_staked, stake_amount);
    assert_eq!(stake_pool_stats.header.last_claimed_offset, 2);
    assert_eq!(stake_pool_stats.balance, 2 * DAILY_INFLATION / 2);

    // Claim rewards
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 4989800 + DAILY_INFLATION / 2);
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 4989800 + 2 * DAILY_INFLATION / 2);

    // Change inflation
    tr.change_inflation(DAILY_INFLATION * 2).await.unwrap();
    let stats = tr.central_state_stats().await.unwrap();
    assert_eq!(stats.daily_inflation, DAILY_INFLATION * 2);

    // Change pool minimum
    tr.change_pool_minimum(&stake_pool_owner, 2000)
        .await
        .unwrap();
    let stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_pool_stats.header.minimum_stake_amount, 2000);
    let staker_account_stats = tr
        .stake_account_stats(staker.pubkey(), stake_pool_owner.pubkey())
        .await
        .unwrap();
    assert_eq!(staker_account_stats.pool_minimum_at_creation, 1000);

    // Change pool multiplier
    tr.change_pool_multiplier(&stake_pool_owner, 60)
        .await
        .unwrap();
    let stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_pool_stats.header.stakers_part, 60);

    // Crank
    tr.sleep(86_400).await.unwrap();
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();
    let stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_pool_stats.header.tag, Tag::StakePool as u8);
    assert_eq!(stake_pool_stats.header.current_day_idx, 3);
    assert_eq!(stake_pool_stats.header.minimum_stake_amount, 2000);
    assert_eq!(stake_pool_stats.header.total_staked, stake_amount);
    assert_eq!(stake_pool_stats.header.last_claimed_offset, 2);
    assert_eq!(stake_pool_stats.balance, 2 * DAILY_INFLATION / 2);

    // Claim pool rewards
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let stake_pool_stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stake_pool_stats.header.tag, Tag::StakePool as u8);
    assert_eq!(stake_pool_stats.header.current_day_idx, 3);
    assert_eq!(stake_pool_stats.header.minimum_stake_amount, 2000);
    assert_eq!(stake_pool_stats.header.total_staked, stake_amount);
    assert_eq!(stake_pool_stats.header.last_claimed_offset, 3);
    assert_eq!(
        stake_pool_stats.balance,
        2 * DAILY_INFLATION / 2 + DAILY_INFLATION * 2 * 2 / 5
    );

    // Claim rewards
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(
        staker_stats.balance,
        4989800 + 2 * DAILY_INFLATION / 2 + DAILY_INFLATION * 2 * 3 / 5
    );
}
