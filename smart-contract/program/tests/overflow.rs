use solana_sdk::signer::Signer;
use solana_test_framework::*;

use crate::common::test_runner::TestRunner;

pub mod common;

mod basic_functionality {
    use super::*;

    #[tokio::test]
    async fn overflow() {
        // Setup the token + basic accounts
        let mut tr = TestRunner::new(1_000_000).await.unwrap();
        // Set daily inflation
        tr.change_inflation(5_479_452_000_000_000).await.unwrap();
        // Create pools
        let pool_owner = tr.create_user_with_ata().await.unwrap();
        let pool_owner2 = tr.create_user_with_ata().await.unwrap();
        tr.create_pool(&pool_owner, 1_000_000_000)
            .await
            .unwrap();
        tr.create_pool(&pool_owner2, 1_000_000_000)
            .await
            .unwrap();
        tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();
        tr.activate_stake_pool(&pool_owner2.pubkey()).await.unwrap();
        // Create a staker
        let staker = tr.create_user_with_ata().await.unwrap();
        tr.mint(&staker.pubkey(), 6_000_000_000_000_000_000)
            .await
            .unwrap();
        tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey())
            .await
            .unwrap();
        tr.create_stake_account(&pool_owner2.pubkey(), &staker.pubkey())
            .await
            .unwrap();
        // Stake
        tr.stake(&pool_owner.pubkey(), &staker, 530_959_347_000_000)
            .await
            .unwrap();
        tr.stake(&pool_owner2.pubkey(), &staker, 704_776_720_000_000)
            .await
            .unwrap();
        // Wait 1 day
        tr.sleep(86400).await.unwrap();
        // Crank
        tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
        // Pool claim
        tr.claim_pool_rewards(&pool_owner).await.unwrap();
        // check pool owner balance
        let owner_stats = tr.staker_stats(pool_owner.pubkey()).await.unwrap();
        assert_eq!(owner_stats.balance, 1177179469601839)
    }
}
