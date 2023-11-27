use solana_program::clock::SECONDS_PER_DAY;
use solana_sdk::signer::Signer;

use access_protocol::state::Tag::BondV2Account;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn v2_bonds() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();

    // ---------------------------------------------------------------------------------------------
    // V1 bond creation should be deprecated
    // ---------------------------------------------------------------------------------------------
    {
        let pool_owner = tr.create_user_with_ata().await.unwrap();
        let bond_creator = tr.create_user_with_ata().await.unwrap();
        tr.create_bond(
            &pool_owner.pubkey(),
            &bond_creator.pubkey(),
            10_000,
            1,
            1,
            1,
        )
            .await
            .unwrap_err();
    }
    // ---------------------------------------------------------------------------------------------
    // Unlockable bond
    // ---------------------------------------------------------------------------------------------
    {
        // Create users
        let pool_owner = tr.create_user_with_ata().await.unwrap();
        let bond_creator = tr.create_user_with_ata().await.unwrap();
        let bond_recipient = tr.create_user_with_ata().await.unwrap();
        // Mint to staker
        tr.mint(&bond_creator.pubkey(), 100_000).await.unwrap();
        // Create stake pool
        tr.create_pool(&pool_owner.pubkey(), 10_000)
            .await
            .unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();

        let current_time = tr.get_current_time().await;
        let unlock_date = current_time + 5 * SECONDS_PER_DAY as i64;
        let bond_amount = 20_000;

        let token_stats = tr.token_stats().await.unwrap();
        let prev_supply = token_stats.supply;

        // Create bond
        tr.create_bond_v2(
            &bond_recipient.pubkey(),
            &pool_owner.pubkey(),
            Some(unlock_date),
        )
            .await
            .unwrap();

        // Add to bond
        tr.add_to_bond_v2(
            &bond_creator,
            &bond_recipient.pubkey(),
            &pool_owner.pubkey(),
            bond_amount,
            Some(unlock_date),
        )
            .await
            .unwrap();

        let staker_stats = tr.staker_stats(bond_creator.pubkey()).await.unwrap();
        assert_eq!(
            staker_stats.balance,
            (100_000 - bond_amount) - (bond_amount as f64 * tr.get_protocol_fees().await / 100.0).ceil() as u64
        );
        let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
        assert_eq!(pool_stats.header.total_staked, bond_amount);
        assert_eq!(pool_stats.vault, bond_amount);
        let token_stats = tr.token_stats().await.unwrap();
        assert_eq!(token_stats.supply, prev_supply);
        let central_state_stats = tr.central_state_stats().await.unwrap();
        assert_eq!(central_state_stats.account.total_staked, bond_amount);
        let bond = tr
            .bond_v2_stats(
                bond_recipient.pubkey(),
                pool_owner.pubkey(),
                Some(unlock_date),
            )
            .await
            .unwrap();
        assert_eq!(bond.tag, BondV2Account);
        assert_eq!(bond.unlock_timestamp, Some(unlock_date));
        assert_eq!(bond.pool, tr.get_pool_pda(&pool_owner.pubkey()));
        assert_eq!(bond.amount, bond_amount);
        assert_eq!(bond.owner, bond_recipient.pubkey());
        assert_eq!(bond.last_claimed_offset, 0);
        assert_eq!(bond.pool_minimum_at_creation, 10_000);

        // Add to bond
        let add_amount = 10;
        tr.add_to_bond_v2(
            &bond_creator,
            &bond_recipient.pubkey(),
            &pool_owner.pubkey(),
            add_amount,
            Some(unlock_date),
        )
            .await
            .unwrap();

        let staker_stats = tr.staker_stats(bond_creator.pubkey()).await.unwrap();
        assert_eq!(
            staker_stats.balance,
            100_000 - (bond_amount + add_amount) - ((bond_amount + add_amount) as f64 * tr.get_protocol_fees().await / 100.0).ceil() as u64
        );
        let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
        assert_eq!(pool_stats.header.total_staked, (bond_amount + add_amount));
        assert_eq!(pool_stats.vault, (bond_amount + add_amount));
        let central_state_stats = tr.central_state_stats().await.unwrap();
        assert_eq!(central_state_stats.account.total_staked, (bond_amount + add_amount));
        let bond = tr
            .bond_v2_stats(
                bond_recipient.pubkey(),
                pool_owner.pubkey(),
                Some(unlock_date),
            )
            .await
            .unwrap();
        assert_eq!(bond.tag, BondV2Account);
        assert_eq!(bond.unlock_timestamp, Some(unlock_date));
        assert_eq!(bond.pool, tr.get_pool_pda(&pool_owner.pubkey()));
        assert_eq!(bond.amount, (bond_amount + add_amount));
        assert_eq!(bond.owner, bond_recipient.pubkey());
        assert_eq!(bond.last_claimed_offset, 0);
        assert_eq!(bond.pool_minimum_at_creation, 10_000);

        // Claim zero rewards
        let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
        assert_eq!(recipient_stats.balance, 0);
        tr.claim_bond_v2_rewards(&bond_recipient, &pool_owner.pubkey(), Some(unlock_date))
            .await
            .unwrap_err();
        let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
        assert_eq!(recipient_stats.balance, 0);

        // Claim rewards
        _ = tr.sleep(SECONDS_PER_DAY).await;
        tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
        tr.claim_bond_v2_rewards(&bond_recipient, &pool_owner.pubkey(), Some(unlock_date))
            .await
            .unwrap();
        let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
        assert_eq!(recipient_stats.balance, 500_000);

        // Try unlocking - shouldn't be possible
        tr.unlock_bond_v2_tokens(&bond_recipient, &pool_owner.pubkey(), Some(unlock_date))
            .await
            .unwrap_err();

        // Move 5 days to the future
        _ = tr.sleep(5 * SECONDS_PER_DAY).await;
        tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
        // Unlocking should not be possible before reward claim
        tr.unlock_bond_v2_tokens(&bond_recipient, &pool_owner.pubkey(), Some(unlock_date))
            .await
            .unwrap_err();
        _ = tr.sleep(1).await;
        // Claim rewards
        tr.claim_bond_v2_rewards(&bond_recipient, &pool_owner.pubkey(), Some(unlock_date))
            .await
            .unwrap();
        // Unlocking should be possible now
        tr.unlock_bond_v2_tokens(&bond_recipient, &pool_owner.pubkey(), Some(unlock_date))
            .await
            .unwrap();
        // Check all the stats
        let creator_stats = tr.staker_stats(bond_creator.pubkey()).await.unwrap();
        assert_eq!(
            creator_stats.balance,
            (100_000 - (bond_amount + add_amount)) - ((bond_amount + add_amount) as f64 * tr.get_protocol_fees().await / 100.0).ceil() as u64
        );
        let recipient_stats = tr.staker_stats(bond_recipient.pubkey()).await.unwrap();
        assert_eq!(
            recipient_stats.balance,
            2 * 500_000 + bond_amount + add_amount
        );
        let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
        assert_eq!(pool_stats.header.total_staked, 0);
        assert_eq!(pool_stats.vault, 0);
        let central_state_stats = tr.central_state_stats().await.unwrap();
        assert_eq!(central_state_stats.account.total_staked, 0);
        let bond = tr
            .bond_v2_stats(
                bond_recipient.pubkey(),
                pool_owner.pubkey(),
                Some(unlock_date),
            )
            .await
            .unwrap();
        assert_eq!(bond.tag, BondV2Account);
        assert_eq!(bond.unlock_timestamp, Some(unlock_date));
        assert_eq!(bond.pool, tr.get_pool_pda(&pool_owner.pubkey()));
        assert_eq!(bond.amount, 0);
        assert_eq!(bond.owner, bond_recipient.pubkey());
        assert_eq!(bond.last_claimed_offset, 6);
        assert_eq!(bond.pool_minimum_at_creation, 10_000);

        // Second unlock should not be possible
        _ = tr.sleep(1).await;
        tr.unlock_bond_v2_tokens(&bond_recipient, &pool_owner.pubkey(), Some(unlock_date))
            .await
            .unwrap_err();
    }

    // ---------------------------------------------------------------------------------------------
    // Forever bond
    // ---------------------------------------------------------------------------------------------
    {
        // Create users
        let pool_owner = tr.create_user_with_ata().await.unwrap();
        let bond_creator = tr.create_user_with_ata().await.unwrap();
        let bond_recipient = tr.create_user_with_ata().await.unwrap();
        // Mint to staker
        tr.mint(&bond_creator.pubkey(), 100_000).await.unwrap();
        // Create stake pool
        tr.create_pool(&pool_owner.pubkey(), 10_000)
            .await
            .unwrap();
        // Activate stake pool
        tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();

        let token_stats = tr.token_stats().await.unwrap();
        let prev_supply = token_stats.supply;

        // Create bond
        let bond_amount = 30_000;
        tr.create_bond_v2(
            &bond_recipient.pubkey(),
            &pool_owner.pubkey(),
            None,
        )
            .await
            .unwrap();

        // add to bond
        tr.add_to_bond_v2(
            &bond_creator,
            &bond_recipient.pubkey(),
            &pool_owner.pubkey(),
            bond_amount,
            None,
        )
            .await
            .unwrap();

        let staker_stats = tr.staker_stats(bond_creator.pubkey()).await.unwrap();
        assert_eq!(
            staker_stats.balance,
            100_000 - bond_amount as u64 // no protocol fee for forever bonds
        );
        let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
        assert_eq!(pool_stats.header.total_staked, bond_amount);
        assert_eq!(pool_stats.vault, 0); // it got burned so it didn't get to the vault
        let central_state_stats = tr.central_state_stats().await.unwrap();
        assert_eq!(central_state_stats.account.total_staked, bond_amount);
        let token_stats = tr.token_stats().await.unwrap();
        assert_eq!(token_stats.supply, prev_supply - bond_amount);
        let bond = tr
            .bond_v2_stats(bond_recipient.pubkey(), pool_owner.pubkey(), None)
            .await
            .unwrap();
        assert_eq!(bond.tag, BondV2Account);
        assert_eq!(bond.unlock_timestamp, None);
        assert_eq!(bond.pool, tr.get_pool_pda(&pool_owner.pubkey()));
        assert_eq!(bond.amount, bond_amount);
        assert_eq!(bond.owner, bond_recipient.pubkey());
        assert_eq!(bond.last_claimed_offset, 6);
        assert_eq!(bond.pool_minimum_at_creation, 10_000);

        // Add to bond
        let add_amount = 40_000;
        tr.add_to_bond_v2(
            &bond_creator,
            &bond_recipient.pubkey(),
            &pool_owner.pubkey(),
            add_amount,
            None,
        )
            .await
            .unwrap();

        let staker_stats = tr.staker_stats(bond_creator.pubkey()).await.unwrap();
        assert_eq!(
            staker_stats.balance,
            100_000 - (bond_amount + add_amount) as u64 // no protocol fee for forever bonds
        );
        let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
        assert_eq!(pool_stats.header.total_staked, (bond_amount + add_amount));
        assert_eq!(pool_stats.vault, 0);
        let central_state_stats = tr.central_state_stats().await.unwrap();
        assert_eq!(central_state_stats.account.total_staked, bond_amount + add_amount);
        let bond = tr
            .bond_v2_stats(bond_recipient.pubkey(), pool_owner.pubkey(), None)
            .await
            .unwrap();
        assert_eq!(bond.tag, BondV2Account);
        assert_eq!(bond.unlock_timestamp, None);
        assert_eq!(bond.pool, tr.get_pool_pda(&pool_owner.pubkey()));
        assert_eq!(bond.amount, (bond_amount + add_amount));
        assert_eq!(bond.owner, bond_recipient.pubkey());
        assert_eq!(bond.last_claimed_offset, 6);
        assert_eq!(bond.pool_minimum_at_creation, 10_000);

        // Try unlocking - shouldn't be possible
        tr.unlock_bond_v2_tokens(&bond_recipient, &pool_owner.pubkey(), None)
            .await
            .unwrap_err();
    }
}
