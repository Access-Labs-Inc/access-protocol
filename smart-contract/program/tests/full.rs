use solana_test_framework::*;
use std::error::Error;

use borsh::BorshDeserialize;
use mpl_token_metadata::pda::find_metadata_account;
use solana_program::{pubkey::Pubkey, system_program};
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

use solana_program_test::{processor, ProgramTest};
use solana_sdk::signer::{keypair::Keypair, Signer};
use solana_sdk::sysvar::clock;
use solana_test_framework::*;
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};

use access_protocol::{
    entrypoint::process_instruction,
    instruction::{
        activate_stake_pool, admin_mint,
        claim_pool_rewards, claim_rewards,
        crank, create_central_state, create_stake_account,
        create_stake_pool, stake, unstake,
    },
};
use access_protocol::instruction::{change_central_state_authority, change_inflation, change_pool_minimum, change_pool_multiplier, claim_bond, claim_bond_rewards, create_bond, unlock_bond_tokens};
use access_protocol::state::{BondAccount, CentralState, StakeAccount, StakePoolHeader, Tag};
use crate::common::test_runner::TestRunner;

pub mod common;

const DAY: u64 = 86400;

#[tokio::test]
async fn full_system_test() {
    // ------------------------------------------
    // DAY 1
    //-------------------------------------------
    // Setup the token + basic accounts
    let mut tr = TestRunner::new().await.unwrap();
    // Setup 3 pool owners
    let pool_owner = tr.create_ata_account().await.unwrap();
    let pool_owner2 = tr.create_ata_account().await.unwrap();
    let pool_owner3 = tr.create_ata_account().await.unwrap();
    // Setup 3 airdrop users
    let airdropUser1 = tr.create_ata_account().await.unwrap();
    let airdropUser2 = tr.create_ata_account().await.unwrap();
    let airdropUser3 = tr.create_ata_account().await.unwrap();
    // Setup 2 vesting users
    let vestingUser1 = tr.create_ata_account().await.unwrap();
    let vestingUser2 = tr.create_ata_account().await.unwrap();

    // Setup all the pools
    tr.create_stake_pool(&pool_owner.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_pool(&pool_owner2.pubkey(), 1_000_000_000).await.unwrap();
    tr.create_stake_pool(&pool_owner3.pubkey(), 1_000_000_000).await.unwrap();
    // Activate the pools
    tr.activate_stake_pool(&pool_owner.pubkey()).await.unwrap();
    tr.activate_stake_pool(&pool_owner2.pubkey()).await.unwrap();

    // ------------------------------------------
    // DAY 2
    //-------------------------------------------
    tr.sleep(DAY).await.unwrap();
    // First two airdrop users get airdrop of 100000 tokens to the first pool with the release date on day 10
    tr.create_bond(&pool_owner.pubkey(), &airdropUser1.pubkey(), 100_000_000_000, 8*DAY as i64).await.unwrap();
    tr.create_bond(&pool_owner.pubkey(), &airdropUser2.pubkey(), 100_000_000_000, 8*DAY as i64).await.unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &airdropUser1).await.unwrap();
    tr.claim_bond(&pool_owner.pubkey(), &airdropUser2).await.unwrap();
}

// // check that this raises the totalRewardable and the pool totalUnrewardedAirdrops
// expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(200_000_000_000);
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6);
// totalRewardables.push(200_000_000_000 * 0.6);
//
// // ------------------------------------------
// // DAY 3
// //-------------------------------------------
// console.log("starting day 3");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// // last airdrop user gets airdrop of 100000 tokens to second pool with the release date on day 10
// await token.connect(owner).airdrop(
// poolOwner2.address,
// [airdropUser3.address],
// [100_000_000_000],
// [currentTimestamp + SECONDS_IN_DAY * 7],
// false,
// );
//
// // staker stakes 10000 tokens to each pool - second stake should fail at first, then we mint some more tokens to the staker
// await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
// // print the currentDayIdx from the poolOwner's pool
// await expect(token.connect(staker).stake(poolOwner.address, 10_200_000_000)).to.be.revertedWithCustomError(token, "InsufficientBalance").withArgs(staker.address, poolOwner.address, 0, 10_200_000_000);
// await token.connect(owner).mint(staker.address, 20_400_000_000);
// await token.connect(staker).stake(poolOwner2.address, 10_200_000_000);
// await token.connect(staker).stake(poolOwner3.address, 10_200_000_000);
// expect(await token.balanceOf(staker.address)).to.equal(0);
//
// // check that the totalStaked and totalRewardable and pool stats are correct
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 30_000_000_000);
// expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(200_000_000_000);
// expect((await token.s_pools(poolOwner2.address)).totalUnrewardedAirdrops).to.equal(100_000_000_000);
// expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(10_000_000_000);
// expect((await token.s_pools(poolOwner2.address)).totalStaked).to.equal(10_000_000_000);
// expect((await token.s_pools(poolOwner3.address)).totalStaked).to.equal(10_000_000_000);
// totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 30_000_000_000);
//
// // ------------------------------------------
// // DAY 4
// //-------------------------------------------
// console.log("starting day 4");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// // create vesting airdrop with cliff at day 7 with rewards for 100_000 tokens to both vesting users to pool1 and pool3
// const dailyVestingAmount = 10_000_000_000;
// await token.connect(owner).airdrop(
// poolOwner.address,
// [vestingUser1.address, vestingUser2.address],
// Array(10).fill(dailyVestingAmount),
// [currentTimestamp + SECONDS_IN_DAY * 3, currentTimestamp + SECONDS_IN_DAY * 4, currentTimestamp + SECONDS_IN_DAY * 5, currentTimestamp + SECONDS_IN_DAY * 6, currentTimestamp + SECONDS_IN_DAY * 7, currentTimestamp + SECONDS_IN_DAY * 8, currentTimestamp + SECONDS_IN_DAY * 9, currentTimestamp + SECONDS_IN_DAY * 10, currentTimestamp + SECONDS_IN_DAY * 11, currentTimestamp + SECONDS_IN_DAY * 12],
// true,
// );
// await token.connect(owner).airdrop(
// poolOwner3.address,
// [vestingUser1.address, vestingUser2.address],
// Array(10).fill(dailyVestingAmount),
// [currentTimestamp + SECONDS_IN_DAY * 3, currentTimestamp + SECONDS_IN_DAY * 4, currentTimestamp + SECONDS_IN_DAY * 5, currentTimestamp + SECONDS_IN_DAY * 6, currentTimestamp + SECONDS_IN_DAY * 7, currentTimestamp + SECONDS_IN_DAY * 8, currentTimestamp + SECONDS_IN_DAY * 9, currentTimestamp + SECONDS_IN_DAY * 10, currentTimestamp + SECONDS_IN_DAY * 11, currentTimestamp + SECONDS_IN_DAY * 12],
// true,
// );
//
// // check that the totalStaked, totalRewardable and pool stats are correct
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 30_000_000_000 + 400_000_000_000);
// expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(200_000_000_000);
// expect((await token.s_pools(poolOwner2.address)).totalUnrewardedAirdrops).to.equal(100_000_000_000);
// expect((await token.s_pools(poolOwner3.address)).totalUnrewardedAirdrops).to.equal(0);
// expect((await token.s_pools(poolOwner.address)).totalRewardedAirdrops).to.equal(200_000_000_000);
// expect((await token.s_pools(poolOwner2.address)).totalRewardedAirdrops).to.equal(0);
// expect((await token.s_pools(poolOwner3.address)).totalRewardedAirdrops).to.equal(200_000_000_000);
// expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(10_000_000_000);
// expect((await token.s_pools(poolOwner2.address)).totalStaked).to.equal(10_000_000_000);
// expect((await token.s_pools(poolOwner3.address)).totalStaked).to.equal(10_000_000_000);
// totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 30_000_000_000 + 400_000_000_000);
//
// // ------------------------------------------
// // DAY 5
// //-------------------------------------------
// console.log("starting day 5");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// // staker unstakes 5000 tokens from pool1 - first unstake should fail at first, then he claims his rewards
// expect(await token.balanceOf(staker.address)).to.equal(0);
// await expect(token.connect(staker).unstake(poolOwner.address, 5_000_000_000)).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(staker.address, poolOwner.address);
// await token.connect(staker).claimRewards(poolOwner.address);
// await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);
//
// // check that the totalStaked, totalRewardable and pool stats are correct
// expect(await token.balanceOf(staker.address)).to.equal(
// Math.round(
// 0.4 * 10_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
// 0.4 * 10_000_000_000 * DAILY_INFLATION / totalRewardables[4]
// ) + 5_000_000_000
// );
// expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000);
// expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(200_000_000_000);
// expect((await token.s_pools(poolOwner2.address)).totalUnrewardedAirdrops).to.equal(100_000_000_000);
// expect((await token.s_pools(poolOwner3.address)).totalUnrewardedAirdrops).to.equal(0);
// expect((await token.s_pools(poolOwner.address)).totalRewardedAirdrops).to.equal(200_000_000_000);
// expect((await token.s_pools(poolOwner2.address)).totalRewardedAirdrops).to.equal(0);
// expect((await token.s_pools(poolOwner3.address)).totalRewardedAirdrops).to.equal(200_000_000_000);
// expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(5_000_000_000);
// expect((await token.s_pools(poolOwner2.address)).totalStaked).to.equal(10_000_000_000);
// expect((await token.s_pools(poolOwner3.address)).totalStaked).to.equal(10_000_000_000);
// totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000);
//
// // ------------------------------------------
// // DAY 6
// //-------------------------------------------
// console.log("starting day 6");
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
// await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
// currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
//
// //vestingUser1 should be able to claim his rewards in pool1
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
// })
// })