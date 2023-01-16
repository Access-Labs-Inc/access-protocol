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
    let airdrop_user1 = tr.create_ata_account().await.unwrap();
    let airdrop_user2 = tr.create_ata_account().await.unwrap();
    let airdrop_user3 = tr.create_ata_account().await.unwrap();

    // Setup 2 vesting users
    let vesting_user1 = tr.create_ata_account().await.unwrap();
    let vesting_user2 = tr.create_ata_account().await.unwrap();

    // Setup 3 pool owners
    let pool_owner = tr.create_ata_account().await.unwrap();
    let pool_owner2 = tr.create_ata_account().await.unwrap();
    let pool_owner3 = tr.create_ata_account().await.unwrap();

    // Setup all the pools
    tr.create_stake_pool(&pool_owner.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_pool(&pool_owner2.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_pool(&pool_owner3.pubkey(), 1_000_000_000).await.unwrap();

    // Activate the pools
    tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();
    tr.activate_stake_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.activate_stake_pool(&pool_owner3.pubkey()).await.unwrap();

    // Setup 1 staker with 10_200_000_000 tokens in his account and appropriate stake accounts for all pools
    let staker = tr.create_ata_account().await.unwrap();
    tr.mint(&staker.pubkey(), 10_200_000_000).await.unwrap();
    tr.create_stake_account(&pool_owner.pubkey(), &staker.pubkey()).await.unwrap();
    tr.create_stake_account(&pool_owner2.pubkey(), &staker.pubkey()).await.unwrap();
    tr.create_stake_account(&pool_owner3.pubkey(), &staker.pubkey()).await.unwrap();

    // ------------------------------------------
    // DAY 2
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    // First two airdrop users get airdrop of 100000 tokens to the first pool with the release date on day 10
    tr.create_bond(&pool_owner.pubkey(), &airdrop_user1.pubkey(), 100_000_000_000, 1,  8*DAY as i64, 1).await.unwrap();
    tr.create_bond(&pool_owner.pubkey(), &airdrop_user2.pubkey(), 100_000_000_000, 1,  8*DAY as i64, 1).await.unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &airdrop_user1.pubkey()).await.unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &airdrop_user2.pubkey()).await.unwrap();

    let pool_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool_stats.total_pool_staked, 200_000_000_000);
    let central_state = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state.total_staked, 200_000_000_000);

    // ------------------------------------------
    // DAY 3
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // last airdrop user gets airdrop of 100000 tokens to second pool with the release date on day 10
    tr.create_bond(&pool_owner2.pubkey(), &airdrop_user3.pubkey(), 100_000_000_000, 1,  7*DAY as i64, 1).await.unwrap();
    tr.claim_bond(&pool_owner2.pubkey(), &airdrop_user3.pubkey()).await.unwrap();

    // staker stakes 10000 tokens to each pool - second stake should fail at first, then we mint some more tokens to the staker
    tr.stake(&pool_owner.pubkey(), &staker, 10_000_000_000).await.unwrap();
    tr.stake(&pool_owner2.pubkey(), &staker, 10_000_000_000).await.unwrap_err();
    tr.mint(&staker.pubkey(), 20_400_000_000).await.unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 20_400_000_000);
    tr.sleep(1).await.unwrap(); // so that we are not sending the same transaction twice in the same block
    tr.stake(&pool_owner2.pubkey(), &staker, 10_000_000_000).await.unwrap();
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 10_200_000_000);
    tr.stake(&pool_owner3.pubkey(), &staker, 10_000_000_000).await.unwrap();

    // check that the totalStaked and pool stats are correct
    let pool1_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool1_stats.total_pool_staked, 210_000_000_000);
    let pool2_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
    assert_eq!(pool2_stats.total_pool_staked, 110_000_000_000);
    let pool3_stats = tr.pool_stats(pool_owner3.pubkey()).await.unwrap();
    assert_eq!(pool3_stats.total_pool_staked, 10_000_000_000);
    let central_state = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state.total_staked, 330_000_000_000);

    // ------------------------------------------
    // DAY 4
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // create vesting airdrop with cliff at day 7 with rewards for 100_000 tokens to both vesting users to pool1 and pool3
    tr.create_bond(&pool_owner.pubkey(), &vesting_user1.pubkey(), 10_000_000_000, 10, 3*DAY as i64, DAY as i64).await.unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &vesting_user1.pubkey()).await.unwrap();
    tr.create_bond(&pool_owner.pubkey(), &vesting_user2.pubkey(), 10_000_000_000, 10, 3*DAY as i64, DAY as i64).await.unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &vesting_user2.pubkey()).await.unwrap();
    tr.create_bond(&pool_owner3.pubkey(), &vesting_user1.pubkey(), 10_000_000_010, 10, 3*DAY as i64, DAY as i64).await.unwrap();
    tr.claim_bond(&pool_owner3.pubkey(), &vesting_user1.pubkey()).await.unwrap();
    tr.create_bond(&pool_owner3.pubkey(), &vesting_user2.pubkey(), 10_000_000_010, 10, 3*DAY as i64, DAY as i64).await.unwrap();
    tr.claim_bond(&pool_owner3.pubkey(), &vesting_user2.pubkey()).await.unwrap();

    // check that the totalStaked and pool stats are correct
    let pool1_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    assert_eq!(pool1_stats.total_pool_staked, 230_000_000_000);
    let pool2_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
    assert_eq!(pool2_stats.total_pool_staked, 110_000_000_000);
    let pool3_stats = tr.pool_stats(pool_owner3.pubkey()).await.unwrap();
    assert_eq!(pool3_stats.total_pool_staked, 30_000_000_020);
    let central_state = tr.central_state_stats().await.unwrap();
    assert_eq!(central_state.total_staked, 370_000_000_020);

    // ------------------------------------------
    // DAY 5
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    // staker unstakes 5000 tokens from pool1 - first unstake should fail at first, then he claims his rewards, but just for pool 1
    tr.unstake(&pool_owner.pubkey(), &staker, 5_000_000_000).await.unwrap_err();
    tr.claim_staker_rewards(&pool_owner.pubkey(), &staker).await.unwrap();
    tr.sleep(1).await.unwrap(); // so that we are not sending the same transaction twice in the same block
    tr.unstake(&pool_owner.pubkey(), &staker, 5_000_000_000).await.unwrap();

    // check that the totalStaked and pool stats are correct
    let pool1_stats = tr.pool_stats(pool_owner.pubkey()).await.unwrap();
    // assert_eq!(pool1_stats.total_pool_staked, 225_000_000_000);
    let pool2_stats = tr.pool_stats(pool_owner2.pubkey()).await.unwrap();
    assert_eq!(pool2_stats.total_pool_staked, 110_000_000_000);
    let pool3_stats = tr.pool_stats(pool_owner3.pubkey()).await.unwrap();
    assert_eq!(pool3_stats.total_pool_staked, 30_000_000_020);
    let central_state = tr.central_state_stats().await.unwrap();
    // assert_eq!(central_state.total_staked, 365_000_000_020);
    let staker_stats = tr.staker_stats(staker.pubkey()).await.unwrap();
    assert_eq!(staker_stats.balance, 5_000_000_000 +
        (
            (DAILY_INFLATION as f64 / 330.0 * 10.0 * 0.5) +
            (DAILY_INFLATION as f64 * 10_000_000_000.0 / 370_000_000_020.0 * 0.5)
        ).round() as u64
    );

    // ------------------------------------------
    // DAY 6
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    tr.crank_pool(&pool_owner.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner2.pubkey()).await.unwrap();
    tr.crank_pool(&pool_owner3.pubkey()).await.unwrap();

    //vestingUser1 should be able to claim his rewards in pool1
    tr.claim_bond_rewards(&pool_owner.pubkey(), &vesting_user1).await.unwrap();
}

// ------------------------------------------
// DAY 6
//-------------------------------------------

// expect(await token.balanceOf(vestingUser1.address)).to.equal(0);
// await token.connect(vestingUser1).claimRewards(poolOwner.address);
// expect(await token.balanceOf(vestingUser1.address)).to.equal
// (
// Math.round(
// (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[4]) +
// (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[5])
// )
// );
//
// // no one can claim their airdrops yet
// await expect(token.connect(airdropUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser1.address, poolOwner.address);
// await expect(token.connect(airdropUser2).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser2.address, poolOwner.address);
// await expect(token.connect(airdropUser3).unlockAirdrop(poolOwner2.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser3.address, poolOwner2.address);
// await expect(token.connect(vestingUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(vestingUser1.address, poolOwner.address);
// await expect(token.connect(vestingUser2).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(vestingUser2.address, poolOwner.address);
//
// // check that the totalStaked, totalRewardable are correct
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000);
//
// // claim rewards as a pool1 owner
// await token.connect(poolOwner).claimRewards(poolOwner.address);
// expect(await token.balanceOf(poolOwner.address)).to.equal(
// Math.floor(
// 0.6 * 200_000_000_000 * DAILY_INFLATION / totalRewardables[2] +
// 0.6 * 210_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
// 0.6 * 410_000_000_000 * DAILY_INFLATION / totalRewardables[4] +
// 0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[5]
// )
// )
// totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000);
//
// // ------------------------------------------
// // DAY 7
// //-------------------------------------------
// console.log("starting day 7");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// // the first part of the vesting airdrop should be ready now
// await token.connect(vestingUser1).claimRewards(poolOwner.address);
// await token.connect(vestingUser1).unlockAirdrop(poolOwner.address);
// expect(await token.balanceOf(vestingUser1.address)).to.equal(
// Math.round(
// (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[4]) +
// (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[5]) +
// (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[6])
// ) + 10_000_000_000
// );
// await expect(token.connect(vestingUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(vestingUser1.address, poolOwner.address);
//
// // pool1 owner should be able to claim his rewards again
// await token.connect(poolOwner).claimRewards(poolOwner.address);
// expect(await token.balanceOf(poolOwner.address)).to.equal(
// Math.floor(
// 0.6 * 200_000_000_000 * DAILY_INFLATION / totalRewardables[2] +
// 0.6 * 210_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
// 0.6 * 410_000_000_000 * DAILY_INFLATION / totalRewardables[4] +
// 0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[5] +
// 0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[6]
// )
// )
//
// // check that the totalStaked, totalRewardable are correct
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 10_000_000_000);
// totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 10_000_000_000);
//
// // ------------------------------------------
// // DAY 8
// //-------------------------------------------
// console.log("starting day 8");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// // the second part of the vesting airdrop should be ready now
// await token.connect(vestingUser1).claimRewards(poolOwner.address);
// await token.connect(vestingUser1).unlockAirdrop(poolOwner.address);
// expect(await token.balanceOf(vestingUser1.address)).to.equal(
// Math.round(
// (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[4]) +
// (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[5]) +
// (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[6]) +
// (0.4 * 90_000_000_000 * DAILY_INFLATION / totalRewardables[7])
// ) + 20_000_000_000 + 1 // +1 rounding error - todo investigate
// );
//
// // check that the totalStaked, totalRewardable are correct
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 20_000_000_000);
// totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 20_000_000_000);
//
// // ------------------------------------------
// // DAY 9
// //-------------------------------------------
// console.log("starting day 9");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// // check that the airdropUsers still cannot claim their airdrop
// await expect(token.connect(airdropUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser1.address, poolOwner.address);
// await expect(token.connect(airdropUser2).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser2.address, poolOwner.address);
// await expect(token.connect(airdropUser3).unlockAirdrop(poolOwner2.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser3.address, poolOwner2.address);
//
// // check that the totalStaked, totalRewardable are correct
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 20_000_000_000);
// totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 20_000_000_000);
//
// // ------------------------------------------
// // DAY 10
// //-------------------------------------------
// console.log("starting day 10");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// // revoke the airdrop for vestingUser1 and airdropUser1
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 380_000_000_000);
// await token.connect(owner).revokeAirdrop(poolOwner.address, [vestingUser1.address, airdropUser1.address]);
// expect(await token.s_totalRewardable()).to.equal(100_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 300_000_000_000);
//
// // check that the airdropUser1 and vestingUser1 cannot claim their airdrop
// await expect(token.connect(airdropUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser1.address, poolOwner.address);
// await expect(token.connect(vestingUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(vestingUser1.address, poolOwner.address);
// //try claiming rewards as a vestingUser1
// await expect(token.connect(vestingUser1).claimRewards(poolOwner.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(vestingUser1.address, poolOwner.address);
//
// // check that the airdropUser2 can claim his airdrop
// await token.connect(airdropUser2).unlockAirdrop(poolOwner.address);
// expect(await token.balanceOf(airdropUser2.address)).to.equal(100_000_000_000);
//
// // check that the totalStaked, totalRewardable are correct
// expect(await token.s_totalRewardable()).to.equal(100_000_000_000 * 0.5 + 25_000_000_000 + 300_000_000_000);
//
// //check that the pool1 owner can claim his rewards
// await token.connect(poolOwner).claimRewards(poolOwner.address);
// expect(await token.balanceOf(poolOwner.address)).to.equal(
// Math.floor(
// 0.6 * 200_000_000_000 * DAILY_INFLATION / totalRewardables[2] +
// 0.6 * 210_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
// 0.6 * 410_000_000_000 * DAILY_INFLATION / totalRewardables[4] +
// 0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[5] +
// 0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[6] +
// 0.6 * 395_000_000_000 * DAILY_INFLATION / totalRewardables[7] +
// 0.6 * 385_000_000_000 * DAILY_INFLATION / totalRewardables[8] +
// 0.6 * 385_000_000_000 * DAILY_INFLATION / totalRewardables[9]
// ) - 2 // -2 because of the rounding error - todo maybe fix this
// );
//
// // check that the totalStaked, totalRewardable are correct
// expect(await token.s_totalRewardable()).to.equal(100_000_000_000 * 0.5 + 25_000_000_000 + 300_000_000_000);
// totalRewardables.push(100_000_000_000 * 0.5 + 25_000_000_000 + 300_000_000_000);
//
// // ------------------------------------------
// // DAY 11
// //-------------------------------------------
// console.log("starting day 11");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// // check that the pool1 owner can claim his rewards
// await token.connect(poolOwner).claimRewards(poolOwner.address);
// expect(await token.balanceOf(poolOwner.address)).to.equal(
// Math.floor(
// 0.6 * 200_000_000_000 * DAILY_INFLATION / totalRewardables[2] +
// 0.6 * 210_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
// 0.6 * 410_000_000_000 * DAILY_INFLATION / totalRewardables[4] +
// 0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[5] +
// 0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[6] +
// 0.6 * 395_000_000_000 * DAILY_INFLATION / totalRewardables[7] +
// 0.6 * 385_000_000_000 * DAILY_INFLATION / totalRewardables[8] +
// 0.6 * 385_000_000_000 * DAILY_INFLATION / totalRewardables[9] +
// 0.6 * 105_000_000_000 * DAILY_INFLATION / totalRewardables[10]
// ) - 2 // -2 because of the rounding error - todo maybe fix this
// );
//
// // staker claims all rewards
// expect(await token.balanceOf(staker.address)).to.equal(
// Math.round(
// 0.4 * 10_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
// 0.4 * 10_000_000_000 * DAILY_INFLATION / totalRewardables[4]
// ) + 5_000_000_000
// );
//
// await token.connect(staker).claimRewards(poolOwner.address);
// await token.connect(staker).claimRewards(poolOwner2.address);
// await token.connect(staker).claimRewards(poolOwner3.address);
//
// // unstake it all
// await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);
// await token.connect(staker).unstake(poolOwner2.address, 10_000_000_000);
// await token.connect(staker).unstake(poolOwner3.address, 10_000_000_000);
//
// // claim rewards as a staker - should fail
// expect(await token.balanceOf(staker.address)).to.equal(
// Math.round(
// (0.4 * 10_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[3] +
// (0.4 * 10_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[4] +
// (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[5] +
// (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[6] +
// (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[7] +
// (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[8] +
// (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[9] +
// (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[10]
// ) + 30_000_000_000 + 1 // +1 because of the rounding error - todo maybe fix this
// );
//
// // check that the totalStaked, totalRewardable are correct
// expect(await token.s_totalRewardable()).to.equal(100_000_000_000 * 0.5 + 300_000_000_000);