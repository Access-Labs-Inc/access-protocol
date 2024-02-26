

use solana_sdk::signer::Signer;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn program_freeze() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();

    // Create users
    let stake_pool_owner = tr.create_user_with_ata().await.unwrap();
    let recommender = tr.create_user_with_ata().await.unwrap();
    let staker = tr.create_user_with_ata().await.unwrap();

    let start_time = tr.get_current_time().await;

    // Staker accepts the invitation
    let fee_payer_balance = tr.fee_payer_sol_balance().await.unwrap();
    println!("Fee payer balance: {}", fee_payer_balance);
    tr.create_royalty(
        &staker,
        &recommender.pubkey(),
        1000, // 10 %
        (start_time + 3 * 86_400) as u64,
    )
        .await
        .unwrap();
    let create_royalty_fee = fee_payer_balance - tr.fee_payer_sol_balance().await.unwrap();

    // Pool owner accepts the invitation
    tr.create_royalty(
        &stake_pool_owner,
        &recommender.pubkey(),
        2000, // 20 %
        (start_time + 2 * 86_400 + 100) as u64,
    )
        .await
        .unwrap();

    // Mint
    let fee_payer_balance = tr.fee_payer_sol_balance().await.unwrap();
    println!("Fee payer balance: {}", fee_payer_balance);
    tr.mint(&staker.pubkey(), 10_200).await.unwrap();
    let mint_fee = fee_payer_balance - tr.fee_payer_sol_balance().await.unwrap();
    println!("Mint fee: {}", mint_fee);

    // Create stake pool on day 1
    tr.create_pool(&stake_pool_owner, 10_000)
        .await
        .unwrap();

    // Activate stake pool
    tr.activate_stake_pool(&stake_pool_owner.pubkey())
        .await
        .unwrap();

    // Create stake account
    tr.create_stake_account(&stake_pool_owner.pubkey(), &staker.pubkey())
        .await
        .unwrap();

    // Stake to pool 1
    tr.stake(&stake_pool_owner.pubkey(), &staker, 10_000)
        .await
        .unwrap();

    // Wait for 1 day
    tr.sleep(86400).await.unwrap();
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim rewards
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 450_000);
    let stats = tr.staker_stats(recommender.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 50_000);

    // Claim pool rewards
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 400_000);
    let stats = tr.staker_stats(recommender.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 150_000);

    // Staker closes the royalty account
    let fee_payer_balance = tr.fee_payer_sol_balance().await.unwrap();
    println!("Fee payer balance: {}", fee_payer_balance);
    tr.close_royalty(&staker).await.unwrap();
    let close_royalty_fee = tr.fee_payer_sol_balance().await.unwrap() - fee_payer_balance;
    assert_eq!(create_royalty_fee - 10_000, close_royalty_fee + 10_000);

    // Wait for 1 day
    tr.sleep(86400).await.unwrap();
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim rewards
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 950_000);
    let stats = tr.staker_stats(recommender.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 150_000);

    // Claim pool rewards
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 800_000);
    let stats = tr.staker_stats(recommender.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 250_000);

    // Wait for 1 day
    tr.sleep(86400).await.unwrap();
    tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();

    // Claim staker rewards with a different royalty account
    tr.claim_staker_rewards(&stake_pool_owner.pubkey(), &staker)
        .await
        .unwrap();
    let stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 950_000 + 500_000);
    let stats = tr.staker_stats(recommender.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 250_000);

    // Claim pool rewards - the royalty account should be expired already
    tr.claim_pool_rewards(&stake_pool_owner).await.unwrap();
    let stats = tr.pool_stats(stake_pool_owner.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 800_000 + 500_000);
    let stats = tr.staker_stats(recommender.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 250_000);


    // Create a new staker
    let staker2 = tr.create_user_with_ata().await.unwrap();
    tr.mint(&staker2.pubkey(), 10_000).await.unwrap();

// Create a bond V2 account
    tr.create_bond_v2(
        &staker2.pubkey(),
        &stake_pool_owner.pubkey(),
        None,
    )
        .await
        .unwrap();

    // Add to the bond V2
    // add to bond
    tr.add_to_bond_v2(
        &staker2,
        &staker2.pubkey(),
        &stake_pool_owner.pubkey(),
        10_000,
        None,
    )
        .await
        .unwrap();

    // Create a new royalty account
    tr.create_royalty(
        &staker2,
        &recommender.pubkey(),
        4000, // 40 %
        (start_time + 1000 * 86_400) as u64,
    )
        .await
        .unwrap();

    // Wait for 10 days
    for _ in 0..10 {
        tr.sleep(86400).await.unwrap();
        tr.crank_pool(&stake_pool_owner.pubkey()).await.unwrap();
    }

    // Claim staker 2 rewards
    tr.claim_bond_v2_rewards(&staker2, &stake_pool_owner.pubkey(), None)
        .await
        .unwrap();
    let stats = tr.staker_stats(staker2.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 150_000 * 10);
    let stats = tr.staker_stats(recommender.pubkey()).await.unwrap();
    assert_eq!(stats.balance, 250_000 + 100_000 * 10);
}
