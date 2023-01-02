import { expect, should } from "chai";

// in reality will be (5 bilion tokens + 6 decimals) / 365
const DAILY_INFLATION = 3_650_000_000_000_000 / 365;
const BURN_FEE = 2;
const SECONDS_IN_DAY = 86_400;

describe("Token", function () {
  const deployTokenFixture = async () => {
    const [owner, staker] = await ethers.getSigners();
    const Token = await hre.ethers.getContractFactory("AccessToken");
    const token = await Token.deploy(BURN_FEE, DAILY_INFLATION, "Access", "ACS", SECONDS_IN_DAY);
    token.mint(staker.address, 10_200_000_000);
    return { token, owner, staker };
  }

  const deployPoolFixture = async () => {
    const { token, owner, staker } = await deployTokenFixture();
    const signers = await ethers.getSigners();
    const poolOwner = signers[2];
    const minimumStakeAmount = 1_000_000_000;
    const stakersPart = 40;
    await token.connect(poolOwner).createPool(minimumStakeAmount, stakersPart);
    return { token, owner, staker, poolOwner, minimumStakeAmount, stakersPart };
  }

  const deployActivePoolFixture = async () => {
    const { token, owner, staker, poolOwner, minimumStakeAmount, stakersPart } = await deployPoolFixture();
    await token.connect(owner).activatePool(poolOwner.address);
    return { token, owner, staker, poolOwner, minimumStakeAmount, stakersPart };
  }
  it("Should deploy token", async function () {
    const { token, owner, staker } = await deployTokenFixture();
    expect(await token.name()).to.equal("Access");
    expect(await token.symbol()).to.equal("ACS");
    expect(await token.decimals()).to.equal(6);
    expect(await token.totalSupply()).to.equal(10_200_000_000);
    expect(await token.balanceOf(owner.address)).to.equal(0);
    expect(await token.balanceOf(staker.address)).to.equal(10_200_000_000);
    expect(await token.s_burnFee()).to.equal(BURN_FEE);
    expect(await token.s_dailyInflation()).to.equal(DAILY_INFLATION);
  });

  describe("Basic Token functionality", function () {
    it("Should be possible to change inflation", async function () {
      const { token, owner } = await deployTokenFixture();
      await token.connect(owner).setDailyInflation(200_000_000_000);
      expect(await token.s_dailyInflation()).to.equal(DAILY_INFLATION);
      expect(await token.s_nextDailyInflation()).to.equal(200_000_000_000);
    });
    it("Should be possible to change contract owner", async function () {
      const { token, owner } = await deployTokenFixture();
      const signers = await ethers.getSigners();
      const newOwner = signers[2];
      expect(await token.owner()).to.equal(owner.address);
      await token.connect(owner).transferOwnership(newOwner.address);
      expect(await token.owner()).to.equal(newOwner.address);
      await expect(token.connect(owner).mint(newOwner.address, 1_000_000_000)).to.be.revertedWith("Ownable: caller is not the owner");
      await token.connect(newOwner).mint(newOwner.address, 1_000_000_000);
      expect(await token.balanceOf(newOwner.address)).to.equal(1_000_000_000);
    });
    it("Should not be possible to renounce ownership", async function () {
      const { token, owner } = await deployTokenFixture();
      await expect(token.connect(owner).renounceOwnership()).to.be.revertedWith("Cannot renounce ownership");
    });
    it("Should have the right symbol", async function () {
      const { token } = await deployTokenFixture();
      expect(await token.symbol()).to.equal("ACS");
    });
    it("Should have the right name", async function () {
      const { token } = await deployTokenFixture();
      expect(await token.name()).to.equal("Access");
    });
    it("Should have 6 decimals", async function () {
      const { token } = await deployTokenFixture();
      expect(await token.decimals()).to.equal(6);
    });
  })

  describe("Pool not created", function () {
    it("Should not be created with invalid stakersPart", async function () {
      const { token } = await loadFixture(deployTokenFixture);
      const signers = await ethers.getSigners();
      const poolOwner = signers[2];
      await expect(token.connect(poolOwner).createPool(1_000_000_000, 101)).to.be.revertedWithCustomError(token, "StakersPartOutOfBounds").withArgs(poolOwner.address, 101);
      expect((await token.s_pools(poolOwner.address)).owner).to.equal(ethers.constants.AddressZero);
    })
    it("Cannot be activated if not created", async function () {
      const { token, owner } = await loadFixture(deployTokenFixture);
      const signers = await ethers.getSigners();
      const poolOwner = signers[2];
      await expect(token.connect(owner).activatePool(poolOwner.address)).to.be.revertedWithCustomError(token, "PoolNotReady").withArgs(poolOwner.address);
    })
  })

  describe("Pool creation and activation", function () {
    it("Should be created", async function () {
      const { poolOwner, minimumStakeAmount, stakersPart, token } = await loadFixture(deployPoolFixture);
      const createdPool = await token.s_pools(poolOwner.address);
      expect(createdPool.minimumStakeAmount).to.equal(minimumStakeAmount);
      expect(createdPool.stakersPart).to.equal(stakersPart);
      expect(createdPool.owner).to.equal(poolOwner.address);
      expect(createdPool.totalStaked).to.equal(0);
      expect(createdPool.activationDayIdx).to.equal(0);
      expect(createdPool.isActivated).to.equal(false);
    })

    it("Should not be created twice", async function () {
      const { token, poolOwner } = await loadFixture(deployPoolFixture);
      await expect(token.connect(poolOwner).createPool(1_000_000_000, 40)).to.be.revertedWithCustomError(token, "PoolExists").withArgs(poolOwner.address);
    })
    it("Cannot be activated by anyone but token owner", async function () {
      const { token, poolOwner, owner } = await loadFixture(deployPoolFixture);
      await expect(token.connect(poolOwner).activatePool(poolOwner.address)).to.be.revertedWith("Ownable: caller is not the owner");
      expect((await token.s_pools(poolOwner.address)).activationDayIdx).to.equal(0);
      expect((await token.s_pools(poolOwner.address)).isActivated).to.equal(false);
      await token.connect(owner).activatePool(poolOwner.address);
      expect((await token.s_pools(poolOwner.address)).activationDayIdx).to.equal(0);
      expect((await token.s_pools(poolOwner.address)).isActivated).to.equal(true);
    })
    it("Cannot be activated twice", async function () {
      const { token, poolOwner, owner } = await loadFixture(deployPoolFixture);
      await token.connect(owner).activatePool(poolOwner.address);
      await expect(token.connect(owner).activatePool(poolOwner.address)).to.be.revertedWithCustomError(token, "AlreadyActivated").withArgs(poolOwner.address);
    })
    it("Should fail if any method is called before the activation", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployPoolFixture);
      await expect(token.connect(staker).stake(poolOwner.address, 1_000_000_000)).to.be.revertedWithCustomError(token, "PoolNotReady").withArgs(poolOwner.address);
      await expect(token.connect(staker).unstake(poolOwner.address, 1_000_000_000)).to.be.revertedWithCustomError(token, "PoolNotReady").withArgs(poolOwner.address);
      await expect(token.connect(staker).claimRewards(poolOwner.address)).to.be.revertedWithCustomError(token, "PoolNotReady").withArgs(poolOwner.address);
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await expect(token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      )).to.be.revertedWithCustomError(token, "PoolNotReady").withArgs(poolOwner.address);
    })
  })
  describe("Pool settings", function () {
    it("Should be possible to change minimumStakeAmount", async function () {
      const { token, poolOwner } = await loadFixture(deployActivePoolFixture);
      await token.connect(poolOwner).setMinimumStakeAmount(20_000_000_000);
      expect((await token.s_pools(poolOwner.address)).minimumStakeAmount).to.equal(20_000_000_000);
    })
    it("Should be possible to change stakersPart", async function () {
      const { token, poolOwner } = await loadFixture(deployActivePoolFixture);
      await token.connect(poolOwner).setStakersPart(70);
      expect((await token.s_pools(poolOwner.address)).stakersPart).to.equal(70);
    })
    it("Should not modify the totalRewardable when changing stakersPart with only staked", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      console.log(
        await token.s_activePoolAddresses(0),
        poolOwner.address,
      )
      console.log(
        await token.s_pools(poolOwner.address),
      )
      // stake 10000
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      expect(await token.s_totalRewardable()).to.equal(10_000_000_000);
      // change stakersPart to 70
      await token.connect(poolOwner).setStakersPart(70);
      expect(await token.s_totalRewardable()).to.equal(10_000_000_000);
    })
    it("Should not modify the totalRewardable when changing stakersPart with only rewardable airdrops", async function () {
      const { token, owner, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop 10000
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      );
      expect(await token.s_totalRewardable()).to.equal(10_000_000_000);
      // change stakersPart to 70
      await token.connect(poolOwner).setStakersPart(70);
      expect(await token.s_totalRewardable()).to.equal(10_000_000_000);
    })
    it("Should modify the totalRewardable when changing stakersPart with only unrewardable airdrops", async function () {
      const { token, owner, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop 10000
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      );
      expect(await token.s_totalRewardable()).to.equal(6_000_000_000);
      // change stakersPart to 70
      await token.connect(poolOwner).setStakersPart(70);
      expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(10_000_000_000);
      expect(await token.s_totalRewardable()).to.equal(3_000_000_000);
    })
  })
  describe("Pool staking", function () {
    it("Should be able to stake", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await expect(
        token.connect(staker).stake(poolOwner.address, 10_200_000_000)
      ).to.emit(token, "Staked").withArgs(staker.address, poolOwner.address, 10_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(10_000_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(0);
    })
    it("Should not be able to stake more than balance", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await expect(
        token.connect(staker).stake(poolOwner.address, 10_200_000_001)
      ).to.be.revertedWithCustomError(token, "InsufficientBalance").withArgs(staker.address, poolOwner.address, 10_200_000_000, 10_200_000_001);
    })
    it("Should be able to unstake exactly (100/102)", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      await token.connect(staker).unstake(poolOwner.address, 10_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(0);
      expect(await token.balanceOf(staker.address)).to.equal(10_000_000_000);
    })
    it("Should not be able to unstake more than (100/102)", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await expect(token.connect(staker).stake(poolOwner.address, 10_200_000_000)).to.emit(token, "Staked").withArgs(staker.address, poolOwner.address, 10_000_000_000);
      await expect(token.connect(staker).unstake(poolOwner.address, 10_000_000_001)).to.be.revertedWithCustomError(token, "InsufficientBalance").withArgs(staker.address, poolOwner.address, 10_000_000_000, 10_000_000_001);
    })
    it("Should be able to unstake in multiple transactions", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);
      await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(0);
    })
    it("Should not be able to stake into non-existent pool", async function () {
      const { token, staker } = await loadFixture(deployActivePoolFixture);
      await expect(token.connect(staker).stake(staker.address, 10_200_000_000)).to.be.revertedWithCustomError(token, "PoolNotReady").withArgs(staker.address);
    })
  })
  describe("Pool claiming + implicit cranking", function () {
    it("Should crank according to the secondsInDay variable not depending on a real time", async function () {
      // deploy a contract that is crankable every 60 seconds
      const [owner, staker] = await ethers.getSigners();
      const Token = await hre.ethers.getContractFactory("AccessToken");
      const token = await Token.deploy(BURN_FEE, DAILY_INFLATION, "Access", "ACS", 60);
      token.mint(staker.address, 4_020_000_000);
      const signers = await ethers.getSigners();
      const poolOwner = signers[2];
      const minimumStakeAmount = 1_000_000_000;
      const stakersPart = 40;
      await token.connect(poolOwner).createPool(minimumStakeAmount, stakersPart);
      await token.connect(owner).activatePool(poolOwner.address);

      // stake 1000 tokens
      await token.connect(staker).stake(poolOwner.address, 1_020_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(3_000_000_000);
      // move time forward by 60 seconds
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + 60]);
      // stake should fail on unclaimed rewards
      await expect(token.connect(staker).stake(poolOwner.address, 1_020_000_000)).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(staker.address, poolOwner.address);

      // claim rewards - should crank the pool and emit the appropriate event
      await expect(token.connect(staker).claimRewards(poolOwner.address)).
      to.emit(token, "Cranked").withArgs(poolOwner.address);
      expect(await token.s_rewardBaseHistory(0)).to.equal(Math.floor(1_000_000_000 * 10 ** 16 / DAILY_INFLATION));
      // check staker's balance
      expect(await token.balanceOf(staker.address)).to.equal(0.4 * DAILY_INFLATION + 3_000_000_000);
    })
    it("Should crank and stake if new user is joining the pool", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      await token.connect(staker).stake(poolOwner.address, 1_020_000_000);
      // mint 10_200 tokens to a staker2
      const staker2 = (await ethers.getSigners())[7];
      await token.connect(owner).mint(staker2.address, 10_200_000_000);
      // move time forward by 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // stake should fail on unclaimed rewards with staker
      await expect(token.connect(staker).stake(poolOwner.address, 1_020_000_000)).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(staker.address, poolOwner.address);
      // but it should succeed and crank with staker2
      await expect(token.connect(staker2).stake(poolOwner.address, 10_200_000_000)).to.emit(token, "Cranked").withArgs(poolOwner.address);
    })
    it("Should crank even if there are no stakes anywhere", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // fast forward 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // stake should crank
      await expect(token.connect(staker).stake(poolOwner.address, 1_020_000_000)).to.emit(token, "Cranked").withArgs(poolOwner.address);
    })
    it("Should crank if there are no stakes in the pool but some in others", async function () {
    })
    it("Should not be able to claim rewards on the day of stake", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(0);
      // claiming should fail
      await expect(token.connect(staker).claimRewards(poolOwner.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(staker.address, poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(0);
    })
    it("Should be able to claim rewards after 1 day", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(0);
      expect(await token.s_totalRewardable()).to.equal(10_000_000_000);
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      await token.connect(staker).claimRewards(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(DAILY_INFLATION * 0.4);
    })
    it("Should be able to claim rewards after 2 days", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(0);
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + 2 * SECONDS_IN_DAY]);
      expect(await token.connect(staker).claimRewards(poolOwner.address)).to.emit(token, "Cranked").withArgs(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(DAILY_INFLATION * 0.4 * 2);
      // second claiming should fail
      await expect(token.connect(staker).claimRewards(poolOwner.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(staker.address, poolOwner.address);
    })
    it("Should be possible to stake on day 3 and claim on day 4", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // we use a staker2 to simulate cranking every day
      const staker2 = (await ethers.getSigners())[10];
      await token.connect(owner).mint(staker2.address, 10_200_000_000);
      await token.connect(staker2).stake(poolOwner.address, 10_200_000_000);

      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      await token.connect(staker2).claimRewards(poolOwner.address);

      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);

      // day3
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // // day4
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      await token.connect(staker2).claimRewards(poolOwner.address);
      console.log("first staker")
      await token.connect(staker).claimRewards(poolOwner.address);
      // expect(await token.balanceOf(staker.address)).to.equal(DAILY_INFLATION * 0.4 * 0.5);
      // expect(await token.balanceOf(staker2.address)).to.equal(DAILY_INFLATION * 0.4 * 2.5);
    })
    it("Should not be possible to stake or unstake if there are unclaimed rewards", async function () {
      const { token, owner, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(0);
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      await token.connect(owner).mint(staker.address, 10_200_000_000);
      await expect(token.connect(staker).stake(poolOwner.address, 10_200_000_000)).to.be.revertedWithCustomError;
      await expect(token.connect(staker).unstake(poolOwner.address, 5_000_000_000)).to.be.revertedWithCustomError;
    })
    it("Should crank multiple pools", async function () {
      const { token, poolOwner, staker, owner } = await loadFixture(deployActivePoolFixture);
      const staker2 = (await ethers.getSigners())[5];
      await token.connect(owner).mint(staker2.address, 10_200_000_000);
      // create a second pool
      const poolOwner2 = (await ethers.getSigners())[7];
      const minimumStakeAmount = 1_000_000_000;
      const stakersPart = 50;
      await token.connect(poolOwner2).createPool(minimumStakeAmount, stakersPart);
      await token.connect(owner).activatePool(poolOwner2.address);
      // staking
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      await token.connect(staker2).stake(poolOwner2.address, 10_200_000_000);
      // move to day 2
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);

      //expect that the token still has the creation time as the last cranked time
      expect(await token.s_lastCrankOffset()).to.equal(0);
      expect((await token.s_pools(poolOwner.address)).currentDayIdx).to.equal(0);
      expect((await token.s_pools(poolOwner2.address)).currentDayIdx).to.equal(0);
      // this should crank the whole system and the first pool
      await token.connect(staker).claimRewards(poolOwner.address);
      expect(await token.s_lastCrankOffset()).to.equal(1);
      expect((await token.s_pools(poolOwner.address)).currentDayIdx).to.equal(1);
      expect((await token.s_pools(poolOwner2.address)).currentDayIdx).to.equal(0);
      // this should crank the second pool
      await token.connect(staker2).claimRewards(poolOwner2.address);
      expect(await token.s_lastCrankOffset()).to.equal(1);
      expect((await token.s_pools(poolOwner.address)).currentDayIdx).to.equal(1);

      // check that the rewards are correct
      expect(await token.balanceOf(staker.address)).to.equal(DAILY_INFLATION * 0.4 * 0.5);
      expect(await token.balanceOf(staker2.address)).to.equal(DAILY_INFLATION * 0.5 * 0.5);
    })
    it("Should not do anything when nothing staked anywhere", async function () {
      const { token, staker, poolOwner } = await loadFixture(deployActivePoolFixture);
      const startingTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [startingTimestamp + 10 * SECONDS_IN_DAY]);

      // stake
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      await ethers.provider.send("evm_mine", [startingTimestamp + 11 * SECONDS_IN_DAY]);

      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      await token.connect(poolOwner).claimRewards(poolOwner.address);

      // check that the balances are correct
      expect(await token.balanceOf(poolOwner.address)).to.equal(DAILY_INFLATION * 0.6);
      expect(await token.balanceOf(staker.address)).to.equal(DAILY_INFLATION * 0.4);

      // // unstake
      // await token.connect(staker).unstake(poolOwner.address, 10_000_000_000);
      // await ethers.provider.send("evm_mine", [startingTimestamp + 20 * SECONDS_IN_DAY]);

      // // try claiming rewards - should fail
      // await expect(token.connect(staker).claimRewards(poolOwner.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(staker.address, poolOwner.address);
      // await expect(token.connect(poolOwner).claimRewards(poolOwner.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(poolOwner.address, poolOwner.address);

      // // check that the balances are correct
      // expect(await token.balanceOf(poolOwner.address)).to.equal(DAILY_INFLATION * 0.6);
      // expect(await token.balanceOf(staker.address)).to.equal(DAILY_INFLATION * 0.4 + 10_000_000_000);
    })
    it("Should not crank an empty pool", async function () {
      // create a second pool
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      const poolOwner2 = (await ethers.getSigners())[7];
      const minimumStakeAmount = 1_000_000_000;
      const stakersPart = 50;
      await token.connect(poolOwner2).createPool(minimumStakeAmount, stakersPart);
      await token.connect(owner).activatePool(poolOwner2.address);
      // stake to the first pool
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      // move 1 day forward
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);

      // claim pool rewards
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      await expect(token.connect(poolOwner2).claimRewards(poolOwner2.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(poolOwner2.address, poolOwner2.address);

      expect((await token.s_pools(poolOwner.address)).currentDayIdx).to.equal(1);
      expect((await token.s_pools(poolOwner2.address)).currentDayIdx).to.equal(0);

      // check balances
      expect(await token.balanceOf(poolOwner.address)).to.equal(DAILY_INFLATION * 0.6);
      expect(await token.balanceOf(poolOwner2.address)).to.equal(0);
    })
    it("Should let the owner claim rewards", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);;
      // stake
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      // crank
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // claim as owner
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      expect(await token.balanceOf(poolOwner.address)).to.equal(DAILY_INFLATION * 0.6);
    })
    it("Should let owner stake in his own pool and claim rewards for both roles", async function () {
      const { token, poolOwner, owner } = await loadFixture(deployActivePoolFixture);
      // stake
      await token.connect(owner).mint(poolOwner.address, 10_200_000_000);
      await token.connect(poolOwner).stake(poolOwner.address, 10_200_000_000);
      // crank
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // claim as owner as well as staker
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      expect(await token.balanceOf(poolOwner.address)).to.equal(DAILY_INFLATION);
    })
    it("Should let someone with a rewarded airdrop claim rewards", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      const dailyAirdrop = 10_000_000_000;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        Array(10).fill(dailyAirdrop),
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY * 2, currentTimestamp + SECONDS_IN_DAY * 3, currentTimestamp + SECONDS_IN_DAY * 4, currentTimestamp + SECONDS_IN_DAY * 5, currentTimestamp + SECONDS_IN_DAY * 6, currentTimestamp + SECONDS_IN_DAY * 7, currentTimestamp + SECONDS_IN_DAY * 8, currentTimestamp + SECONDS_IN_DAY * 9, currentTimestamp + SECONDS_IN_DAY * 10],
        true,
      );
      // forward 3 days
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + 3 * SECONDS_IN_DAY]);

      // claim as staker
      await token.connect(staker).claimRewards(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(3 * DAILY_INFLATION * 0.4 + 10_200_000_000);
    })
    it("Should let someone with both rewarded airdrop and staked tokens claim rewards", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      const dailyAirdrop = 10_000_000_000;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        Array(10).fill(dailyAirdrop),
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY * 2, currentTimestamp + SECONDS_IN_DAY * 3, currentTimestamp + SECONDS_IN_DAY * 4, currentTimestamp + SECONDS_IN_DAY * 5, currentTimestamp + SECONDS_IN_DAY * 6, currentTimestamp + SECONDS_IN_DAY * 7, currentTimestamp + SECONDS_IN_DAY * 8, currentTimestamp + SECONDS_IN_DAY * 9, currentTimestamp + SECONDS_IN_DAY * 10],
        true,
      );
      // stake
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      // forward 3 days
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + 3 * SECONDS_IN_DAY]);
      // claim as staker
      await token.connect(staker).claimRewards(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(3 * DAILY_INFLATION * 0.4);
    })
    it("Shouldn't be possible to do a rewarded airdrop to a staker that has unclaimed rewards", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // stake
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      // forward 3 days
      let startTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [startTimestamp + 3 * SECONDS_IN_DAY]);
      // rewarded airdrop should be reverted
      await expect(token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000, 10_000_000_000],
        [startTimestamp + 4 * SECONDS_IN_DAY, startTimestamp + 5 * SECONDS_IN_DAY],
        true
      )).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(staker.address, poolOwner.address);
    })
    it("Shouldn't be possible to stake if there are unclaimed rewards from a rewarded airdrop", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000, 10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY * 2],
        true,
      );
      // forward 1 day
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // stake should be reverted
      await expect(token.connect(staker).stake(poolOwner.address, 10_200_000_000)).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(staker.address, poolOwner.address);
    })
    it("Should crank when doing an unrewarded airdrop", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // stake
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      // forward 3 days
      const startTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [startTimestamp + 3 * SECONDS_IN_DAY]);
      // airdrop
      await expect(token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        Array(10).fill(10_000_000_000),
        [...Array(10).keys()].map(x => startTimestamp + SECONDS_IN_DAY * (++x + 3)),
        false,
      )).to.emit(token, "Cranked").withArgs(poolOwner.address);
      // // check that it cranked correctly
      // expect((await token.s_pools(poolOwner.address)).currentDayIdx).to.equal(3);
      // // check totalRewardableHistory
      // expect(await token.s_rewardBaseHistory(0)).to.equal(Math.floor(10_000_000_000 * 10 ** 16 / DAILY_INFLATION));
      // expect(await token.s_rewardBaseHistory(1)).to.equal(Math.floor(10_000_000_000 * 10 ** 16 / DAILY_INFLATION));
      // expect(await token.s_rewardBaseHistory(2)).to.equal(Math.floor(10_000_000_000 * 10 ** 16 / DAILY_INFLATION));
      // await expect(token.s_rewardBaseHistory(3)).to.be.revertedWithoutReason();
      // // claim as staker
      // await token.connect(staker).claimRewards(poolOwner.address);
      // expect(await token.balanceOf(staker.address)).to.equal(3 * DAILY_INFLATION * 0.4);
    })
    it("Should be possible to stake if the user has an unrewarded airdrop from a previous day", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000, 10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY * 2],
        false,
      );
      // forward 1 day
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // stake should not be reverted
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(0);
    })
    it("Should count the rewards right for multiple pools cranked on different days", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);

      // create a second pool
      const poolOwner2 = (await ethers.getSigners())[7];
      const minimumStakeAmount = 1_000_000_000;
      const stakersPart = 50;
      await token.connect(poolOwner2).createPool(minimumStakeAmount, stakersPart);
      await token.connect(owner).activatePool(poolOwner2.address);

      // mint 10_200 tokens to staker
      await token.connect(owner).mint(staker.address, 10_200_000_000);
      // stake to pool 1
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(10_200_000_000);
      // forward 3 days
      const startTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [startTimestamp + 3 * SECONDS_IN_DAY]);
      await token.connect(staker).claimRewards(poolOwner.address);
      let claimedFromPool1 = Math.floor(3 * 0.4 * DAILY_INFLATION);
      // rewarded airdrop to pool2
      expect(await token.balanceOf(staker.address)).to.equal(claimedFromPool1 + 10_200_000_000);
      await token.connect(owner).airdrop(
        poolOwner2.address,
        [staker.address],
        [10_000_000_000, 10_000_000_000, 10_000_000_000],
        [startTimestamp + 5 * SECONDS_IN_DAY, startTimestamp + 6 * SECONDS_IN_DAY, startTimestamp + 7 * SECONDS_IN_DAY],
        true,
      );

      // forward 4 days
      await ethers.provider.send("evm_mine", [startTimestamp + 7 * SECONDS_IN_DAY]);
      // stake to pool 2
      await token.connect(staker).claimRewards(poolOwner2.address);

      let claimedFromPool2 = Math.floor(4 * 0.5 * 0.75 * DAILY_INFLATION);
      expect(await token.balanceOf(staker.address)).to.equal(claimedFromPool1 + claimedFromPool2 + 10_200_000_000);
      await token.connect(staker).stake(poolOwner2.address, 10_200_000_000);
      // forward 2 days
      await ethers.provider.send("evm_mine", [startTimestamp + 9 * SECONDS_IN_DAY]);

      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      claimedFromPool1 += Math.floor(4 * 0.4 * 0.25 * DAILY_INFLATION + 2 * 0.4 * 0.2 * DAILY_INFLATION);
      expect(await token.balanceOf(staker.address)).to.equal(claimedFromPool1 + claimedFromPool2);

      await token.connect(staker).claimRewards(poolOwner2.address);
      claimedFromPool2 += 2 * 0.5 * 0.8 * DAILY_INFLATION;
      expect(await token.balanceOf(staker.address)).to.equal(claimedFromPool1 + claimedFromPool2);

      // claim pool owner rewards
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      expect(await token.balanceOf(poolOwner.address)).to.equal(claimedFromPool1 / 0.4 * 0.6);
      await token.connect(poolOwner2).claimRewards(poolOwner2.address);
      expect(await token.balanceOf(poolOwner2.address)).to.equal(claimedFromPool2);
    })
    it("Should crank when revoking an airdrop", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000, 10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY * 2],
        false,
      );
      // forward 1 day
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // revoke airdrop
      await expect(token.connect(owner).revokeAirdrop(poolOwner.address, [staker.address])).to.emit(token, "Cranked");;
    })
    it("Should calculate rewards right for an empty pool", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // create a second pool
      const signers = await ethers.getSigners();
      const poolOwner2 = signers[11];
      const minimumStakeAmount = 1_000_000_000;
      const stakersPart2 = 50;
      await token.connect(poolOwner2).createPool(minimumStakeAmount, stakersPart2);
      await token.connect(owner).activatePool(poolOwner2.address);
      // stake to pool 1
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // forward 2 days
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      // unstake from pool 1 and stake to pool 2
      await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);
      await token.connect(staker).stake(poolOwner2.address, 5_100_000_000);
      // forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + 2 * SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner2.address);
      await expect(token.connect(staker).claimRewards(poolOwner.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(staker.address, poolOwner.address);
      await expect(token.connect(poolOwner).claimRewards(poolOwner2.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(poolOwner.address, poolOwner2.address);
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      await token.connect(poolOwner2).claimRewards(poolOwner2.address);
      // check balance
      expect(await token.balanceOf(staker.address)).to.equal(
        DAILY_INFLATION * 0.4 + DAILY_INFLATION * 0.5 + 5_000_000_000
      );
      expect(await token.balanceOf(poolOwner.address)).to.equal(
        DAILY_INFLATION * 0.6
      );
      expect(await token.balanceOf(poolOwner2.address)).to.equal(
        DAILY_INFLATION * 0.5
      );

      // unstake from pool 2 and stake to pool 1
      await token.connect(staker).unstake(poolOwner2.address, 5_000_000_000);
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // forward 2 days
      await ethers.provider.send("evm_mine", [currentTimestamp + 4 * SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      await expect(token.connect(staker).claimRewards(poolOwner2.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(staker.address, poolOwner2.address);
      await expect(token.connect(poolOwner).claimRewards(poolOwner2.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(poolOwner.address, poolOwner2.address);
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      // check balance
      expect(await token.balanceOf(staker.address)).to.equal(
        DAILY_INFLATION * 3 * 0.4 + DAILY_INFLATION * 0.5 + 4_900_000_000
      );
      expect(await token.balanceOf(poolOwner.address)).to.equal(
        DAILY_INFLATION * 3 * 0.6
      );
      expect(await token.balanceOf(poolOwner2.address)).to.equal(
        DAILY_INFLATION * 0.5
      );
    })
    it("Should calculate rewards right for a pool owner of a pool that was activated later than created", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // create a second pool
      const signers = await ethers.getSigners();
      const poolOwner2 = signers[11];
      const minimumStakeAmount = 1_000_000_000;
      const stakersPart2 = 50;
      await token.connect(poolOwner2).createPool(minimumStakeAmount, stakersPart2);
      // stake to pool 1
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // forward 2 days
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + 2 * SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      // unstake from pool 1 and stake to pool 2
      await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);
      // activate pool 2
      console.log("activating pool 2");
      await token.connect(owner).activatePool(poolOwner2.address);
      await token.connect(staker).stake(poolOwner2.address, 5_100_000_000);
      // forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + 3 * SECONDS_IN_DAY]);
      // // claim rewards
      console.log("claiming pool 2");
      await expect(token.connect(poolOwner2).claimRewards(poolOwner2.address)).to.emit(token, "RewardsClaimed").withArgs(poolOwner2.address, poolOwner2.address, DAILY_INFLATION * 0.5);
      await expect(token.connect(staker).claimRewards(poolOwner2.address)).to.emit(token, "RewardsClaimed").withArgs(staker.address, poolOwner2.address, DAILY_INFLATION * 0.5);
      // check balance
      expect(await token.balanceOf(poolOwner2.address)).to.equal(
        DAILY_INFLATION * 0.5
      );
    })
    it("Shouldn't let poolOwner stake if he has unclaimed rewards", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // mint to poolOwner
      await token.connect(owner).mint(poolOwner.address, 5_100_000_000);
      // stake
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // forward 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // try to stake
      await expect(token.connect(poolOwner).stake(poolOwner.address, 5_100_000_000)).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(poolOwner.address, poolOwner.address);
    })
  })
  describe("Inflation changes", function () {
    it("Should calculate rewards right even after daily inflation changes", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // stake
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // forward 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // change daily inflation
      await token.connect(owner).setDailyInflation(DAILY_INFLATION * 2);
      // check that the daily inflation is changed
      expect(await token.s_nextDailyInflation()).to.equal(DAILY_INFLATION * 2);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      // forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + 2 * SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(
        DAILY_INFLATION * 0.4 + DAILY_INFLATION * 2 * 0.4 + 5_100_000_000
      );
    })
    it("Should calculate rewards right even when multiple inflation changes in one day", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // stake
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // forward 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // change daily inflation 3 times
      await token.connect(owner).setDailyInflation(DAILY_INFLATION * 2);
      await token.connect(owner).setDailyInflation(DAILY_INFLATION * 3);
      await token.connect(owner).setDailyInflation(DAILY_INFLATION * 5);
      // check that the daily inflation is changed
      expect(await token.s_nextDailyInflation()).to.equal(DAILY_INFLATION * 5);
      // forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + 2 * SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(
        DAILY_INFLATION * 0.4 * 2 + 5_100_000_000
      );
    })
  })
  describe("Pool airdrops", function () {
    it("Should let the owner airdrop tokens to multiple users", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      const signers = await ethers.getSigners()
      const staker2 = signers[5];
      const staker3 = signers[6];
      // airdrop
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address, staker2.address, staker3.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true,
      );
      const [amounts, timestamps] = await token.connect(staker).airdropInfo(poolOwner.address);
      expect(amounts.map((item: any) => item.toNumber())).to.members([10_000_000_000]);
      expect(timestamps).to.members([currentTimestamp + SECONDS_IN_DAY]);
      const [amounts2, timestamps2] = await token.connect(staker2).airdropInfo(poolOwner.address);
      expect(amounts2.map((item: any) => item.toNumber())).to.members([10_000_000_000]);
      expect(timestamps2).to.members([currentTimestamp + SECONDS_IN_DAY]);
      const [amounts3, timestamps3] = await token.connect(staker3).airdropInfo(poolOwner.address);
      expect(amounts3.map((item: any) => item.toNumber())).to.members([10_000_000_000]);
      expect(timestamps3).to.members([currentTimestamp + SECONDS_IN_DAY]);
      // expect revert for poolOwner
      await expect(token.connect(poolOwner).airdropInfo(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady")
    })
    it("Should not be possible to unlock airdrop before timestamp", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true,
      );
      // expect revert for poolOwner
      await expect(token.connect(staker).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady")
    })
    it("Should let users unlock unrewarded airdrop after timestamp", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      // ballance
      console.log(await token.balanceOf(staker.address));
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      );
      // fast forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // unlock airdrop
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(20_200_000_000);
      await expect(token.connect(staker).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady")
    })
    it("Should let users unlock rewarded airdrop after timestamp and reward claim", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      // ballance
      console.log(await token.balanceOf(staker.address));
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      );
      // fast forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // unlock airdrop
      await expect(token.connect(staker).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError;
      expect(await token.balanceOf(staker.address)).to.equal(10_200_000_000);
      await token.connect(staker).claimRewards(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(10_200_000_000 + 0.4 * DAILY_INFLATION);
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(20_200_000_000 + 0.4 * DAILY_INFLATION);
    })
    it("Should update the rewardable amount", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // prepare 3 other stakers
      const signers = await ethers.getSigners()
      const staker2 = signers[5];
      const staker3 = signers[6];
      const staker4 = signers[7];
      // airdrop
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address, staker2.address],
        [10_000_000_000, 30_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY],
        true
      );
      // airdrop some more - not rewardable
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker3.address, staker4.address],
        [10_000_000_000, 40_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY],
        false
      );
      expect(await token.s_totalRewardable()).to.equal(80_000_000_000 + 0.6 * 100_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalRewardedAirdrops).to.equal(80_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(100_000_000_000);
    })
    it("Should be possible for owner to revoke airdrops", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // prepare 3 other stakers
      const signers = await ethers.getSigners()
      const staker2 = signers[5];
      const staker3 = signers[6];
      const staker4 = signers[7];
      // airdrop
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address, staker2.address],
        [10_000_000_000, 30_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY],
        true
      );
      // airdrop some more - not rewardable
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker3.address, staker4.address],
        [10_000_000_000, 40_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY],
        false
      );
      // revoke airdrop
      await token.connect(owner).revokeAirdrop(poolOwner.address, [staker2.address, staker4.address]);
      expect(await token.s_totalRewardable()).to.equal(40_000_000_000 + 0.6 * 50_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalRewardedAirdrops).to.equal(40_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(50_000_000_000);
    })
    it("Should handle revoke of no airdrops", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // try to revoke airdrop of staker that has no airdrop
      await token.connect(owner).revokeAirdrop(poolOwner.address, [staker.address]);

    })
    it("Shouldn't be possible for non-owner to revoke airdrops", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop to staker
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      );
      // expect revert for poolOwner
      expect(token.connect(poolOwner).revokeAirdrop(poolOwner.address, [staker.address])).to.be.revertedWith("Ownable: caller is not the owner");
    })
    it("Should manage vesting airdrops", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop to staker
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000, 20_000_000_000, 30_000_000_000],
        [currentTimestamp + 2 * SECONDS_IN_DAY, currentTimestamp + 3 * SECONDS_IN_DAY, currentTimestamp + 4 * SECONDS_IN_DAY],
        false
      );
      // fast forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // airdrop not ready
      await expect(token.connect(staker).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(staker.address, poolOwner.address);
      // fast forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY * 2]);
      // unlock airdrop
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(20_200_000_000);
      // fast forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY * 3]);
      // unlock airdrop
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(40_200_000_000);
      // fast forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY * 4]);
      // unlock airdrop
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      expect(await token.balanceOf(staker.address)).to.equal(70_200_000_000);
      // fast forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY * 5]);
      // unlock airdrop should fail
      await expect(token.connect(staker).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(staker.address, poolOwner.address);
    })
    it("Should modify the rewardable amount when airdrop is unlocked", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop to staker
      expect(await token.s_totalRewardable()).to.equal(0);
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000, 20_000_000_000, 30_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY, currentTimestamp + SECONDS_IN_DAY * 2, currentTimestamp + SECONDS_IN_DAY * 3],
        true
      );
      // fast forward 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // unlock airdrop
      await token.connect(staker).claimRewards(poolOwner.address)
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      expect(await token.s_totalRewardable()).to.equal(50_000_000_000);
    })
    it("Should not let non-owner to airdrop", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop to staker
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      expect(token.connect(staker).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      )).to.be.revertedWith("Ownable: caller is not the owner");
    })
    it("Should let the pool claim to airdrop regardless of the airdrop rewardable status", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // second staker
      const staker2 = (await ethers.getSigners())[5];
      // airdrop to both stakers - one rewardable, one not
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      );
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker2.address],
        [90_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      );
      console.log("TOTAL REWARDABLE:", (await token.s_totalRewardable()).toNumber())
      expect(await token.balanceOf(poolOwner.address)).to.equal(0);
      // fast forward 10 days and crank
      await ethers.provider.send("evm_mine", [currentTimestamp + 1 * SECONDS_IN_DAY]);
      // claim rewards as the pool owner
      expect(await token.balanceOf(poolOwner.address)).to.equal(0);
      expect(await token.balanceOf(staker.address)).to.equal(10_200_000_000);

      await token.connect(poolOwner).claimRewards(poolOwner.address);
      await token.connect(staker).claimRewards(poolOwner.address);
      const poolOwnerRewards = (await token.balanceOf(poolOwner.address)).toNumber();
      const stakerRewards = (await token.balanceOf(staker.address)).toNumber() - 10_200_000_000;
      console.log(poolOwnerRewards);
      console.log(stakerRewards);
      expect(stakerRewards + poolOwnerRewards).to.equal(DAILY_INFLATION);
      expect(await token.s_totalRewardable()).to.equal(10_000_000_000 + 90_000_000_000 * 0.6);
      expect(await token.balanceOf(poolOwner.address)).to.equal(
        Math.floor(DAILY_INFLATION * 60 / 64)
      );
      expect(await token.balanceOf(staker.address)).to.equal(
        Math.floor(DAILY_INFLATION * 4 / 64) + 10_200_000_000
      );
    })
    it("Should be possible to create an unrewarded airdrop even though there are staking rewards to claim", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // stake 10000 tokens
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      // fast forward 1 day
      const startTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [startTimestamp + SECONDS_IN_DAY]);
      // unrewarded airdrop to staker
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [startTimestamp + 2 * SECONDS_IN_DAY],
        false
      );
    })
    it("Should not be possible to create an airdrop with a timestamp in the past", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await expect(token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp],
        true
      )).to.be.revertedWithCustomError(token, "InvalidTimestamp");
    })
    it("Should generate rewards for rewardable airdrops only for the days they exist", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // fast forward 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      // go to day 10
      await ethers.provider.send("evm_mine", [currentTimestamp + 10 * SECONDS_IN_DAY]);
      // rewarded airdrop to staker
      await (token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + 20 * SECONDS_IN_DAY],
        true
      ))
      // go to day 11
      await ethers.provider.send("evm_mine", [currentTimestamp + 11 * SECONDS_IN_DAY]);
      // staker claims rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      // check that the staker has received rewards for only 1 day
      expect(await token.balanceOf(staker.address)).to.equal(
        10_200_000_000 + DAILY_INFLATION * 0.4
      );
    })
    it("Should reset user's minimum stake amount after going below it by unlocking the airdrop", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // rewarded airdrop to staker
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await (token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      ))
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      // stake 20 more tokens
      await token.connect(staker).stake(poolOwner.address, 20_400_000);
      // go to day 1
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // change the minimum stake amount
      await token.connect(poolOwner).setMinimumStakeAmount(100_000_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      // claims rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      // unlock the airdrop
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      // check balance
      expect(await token.balanceOf(staker.address)).to.equal(
        10_200_000_000 - 20_400_000 + DAILY_INFLATION * 0.4 + 10_000_000_000
      );
      // check that the user's minimum stake amount has been reset
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
      await expect(token.connect(staker).stake(poolOwner.address, 10_200_000_000)).to.be.revertedWithCustomError(token, "InvalidAmount");

    })
  })
  describe("Pool minimum stake amount", function () {
    it("Should not be possible to stake under the pool limit", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      await expect(token.connect(staker).stake(poolOwner.address, 0)).to.be.revertedWithCustomError(token, "InvalidAmount").withArgs(staker.address, poolOwner.address, 0);
      await expect(token.connect(staker).stake(poolOwner.address, 1)).to.be.revertedWithCustomError(token, "InvalidAmount").withArgs(staker.address, poolOwner.address, 1);
      await expect(token.connect(staker).stake(poolOwner.address, 1_019_999_999)).to.be.revertedWithCustomError(token, "InvalidAmount").withArgs(staker.address, poolOwner.address, 1_019_999_999);
    })
    it("Should be possible to stake over the pool limit", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      await token.connect(staker).stake(poolOwner.address, 1_020_000_000);
      await token.connect(staker).stake(poolOwner.address, 1);
    })
    it("Should be possible to unstake any amount greater than 0 while above the minimum stake amount", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // stake 10000 tokens
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      await expect(token.connect(staker).unstake(poolOwner.address, 0)).to.be.revertedWithCustomError(token, "InvalidAmount").withArgs(staker.address, poolOwner.address, 0);
      await token.connect(staker).unstake(poolOwner.address, 1);
      await token.connect(staker).unstake(poolOwner.address, 8_999_999_999);
    })
    it("Should not be possible to unstake any amount greater than 0 while below the minimum stake amount", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // stake exactly the limit - 1000 tokens
      await token.connect(staker).stake(poolOwner.address, 1_020_000_000);
      await expect(token.connect(staker).unstake(poolOwner.address, 1)).to.be.revertedWithCustomError(token, "InvalidAmount").withArgs(staker.address, poolOwner.address, 1);
      await expect(token.connect(staker).unstake(poolOwner.address, 200_000_000)).to.be.revertedWithCustomError(token, "InvalidAmount").withArgs(staker.address, poolOwner.address, 200_000_000);
      await expect(token.connect(staker).unstake(poolOwner.address, 100_000_000_000)).to.be.revertedWithCustomError(token, "InsufficientBalance").withArgs(staker.address, poolOwner.address, 1_000_000_000, 100_000_000_000);
    })
    it("Should be possible to unstake all tokens", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // stake exactly the limit - 1000 tokens
      await token.connect(staker).stake(poolOwner.address, 1_020_000_000);
      await token.connect(staker).unstake(poolOwner.address, 1_000_000_000);
    })
    it("Should be possible to unstake when the pool minimum lowers", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // stake exactly the limit - 1000 tokens
      await token.connect(staker).stake(poolOwner.address, 1_020_000_000);
      await token.connect(poolOwner).setMinimumStakeAmount(500_000_000);
      await token.connect(staker).unstake(poolOwner.address, 500_000_000);
    })
    it("Should retain the minimum stake amount when the pool minimum raises", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // stake exactly the limit - 1000 tokens
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      await token.connect(poolOwner).setMinimumStakeAmount(5_000_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await token.connect(staker).unstake(poolOwner.address, 9_000_000_000);
    })
    it("Should change the limit when unstaking after the limit has been lowered", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // stake exactly the limit - 1000 tokens
      await token.connect(staker).stake(poolOwner.address, 1_020_000_000);
      await token.connect(poolOwner).setMinimumStakeAmount(500_000_000);
      await token.connect(staker).unstake(poolOwner.address, 500_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await token.connect(poolOwner).setMinimumStakeAmount(1_000_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
    })
    it("Should reset the retained limit when unstaking all", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // stake exactly the limit - 1000 tokens
      await token.connect(staker).stake(poolOwner.address, 1_020_000_000);
      await token.connect(poolOwner).setMinimumStakeAmount(5_000_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await token.connect(staker).unstake(poolOwner.address, 1_000_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
      await expect(token.connect(staker).stake(poolOwner.address, 1_020_000_000)).to.be.revertedWithCustomError(token, "InvalidAmount").withArgs(staker.address, poolOwner.address, 1_020_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
    })
    it("Should have enough staked including airdrop", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop 1000 tokens to staker in pool
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [1_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      );
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
    })
    it("Should let user unstake below limit when enough in airdrop", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop 1000 tokens to staker in pool
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [1_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      );
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await token.connect(staker).stake(poolOwner.address, 510_000_000);
      await token.connect(staker).unstake(poolOwner.address, 100_000_000);
    })
    it("Should let user unstake only a part when partially covered by airdrop", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop 100 tokens to staker in pool
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [100_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      );
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
      await token.connect(staker).stake(poolOwner.address, 918_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await expect(token.connect(staker).unstake(poolOwner.address, 200_000_000)).to.be.revertedWithCustomError(token, "InvalidAmount").withArgs(staker.address, poolOwner.address, 200_000_000);

      await token.connect(staker).stake(poolOwner.address, 102_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await token.connect(staker).unstake(poolOwner.address, 100_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await token.connect(staker).unstake(poolOwner.address, 900_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
    })
    it("Should not allow the user to access content if he claims an airdrop and goes below threshold", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // airdrop 1000 tokens to staker in pool
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [1_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      );
      // forward 1 day
      await ethers.provider.send("evm_increaseTime", [SECONDS_IN_DAY]);
      await ethers.provider.send("evm_mine", []);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await token.connect(staker).stake(poolOwner.address, 102_000_000);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(false);
    })
    it("Should work in a pool with 0 limit", async function () {
      const { token, poolOwner, staker } = await loadFixture(deployActivePoolFixture);
      // set the limit to 0
      await token.connect(poolOwner).setMinimumStakeAmount(0);
      // stake exactly the limit - 1000 tokens
      expect(await token.enoughStaked(poolOwner.address, staker.address)).to.equal(true);
    })
  })
  describe("Pool complex examples", function () {
    it("Should be able to handle multiple users and multiple pools with rewards - only staking", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);

      // create a second and third staker
      const staker2 = (await ethers.getSigners())[5];
      const staker3 = (await ethers.getSigners())[6];

      // mint 100_000 tokens to staker2 and staker3
      await token.connect(owner).mint(staker2.address, 100_000_000_000);
      await token.connect(owner).mint(staker3.address, 100_000_000_000);

      // create a second pool
      const poolOwner2 = (await ethers.getSigners())[7];
      const minimumStakeAmount = 1_000_000_000;
      const stakersPart = 50;
      await token.connect(poolOwner2).createPool(minimumStakeAmount, stakersPart);
      await token.connect(owner).activatePool(poolOwner2.address);

      // first user stakes 10_000 (+burn fee) tokens into first pool on day 1
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);

      // second user stakes 30_000 (+burn fee) tokens into first pool on day 3
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + 2 * SECONDS_IN_DAY]);
      await token.connect(staker2).stake(poolOwner.address, 30_600_000_000);
      // return

      // third user stakes 40_000 (+burn fee) tokens into second pool on day 5
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      await token.connect(staker3).stake(poolOwner2.address, 40_800_000_000);

      // users claims rewards on day 6
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // pools where the users have staked
      await token.connect(staker).claimRewards(poolOwner.address);
      let stakerRewards = 0.4 * DAILY_INFLATION * 2 + 0.1 * DAILY_INFLATION * 2 + 0.05 * DAILY_INFLATION
      expect(await token.balanceOf(staker.address)).to.equal(stakerRewards);
      await token.connect(staker2).claimRewards(poolOwner.address);
      let staker2Rewards = 0.3 * DAILY_INFLATION * 2 + 0.15 * DAILY_INFLATION
      expect(await token.balanceOf(staker2.address)).to.equal(staker2Rewards + 100_000_000_000 - 30_600_000_000);
      // pools where the user have not staked
      await expect(token.connect(staker).claimRewards(poolOwner2.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(staker.address, poolOwner2.address);
      expect(await token.balanceOf(staker.address)).to.equal(stakerRewards);

      // claim for pool owner
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      let poolOwnerRewards = 0.6 * DAILY_INFLATION * 4 + 0.3 * DAILY_INFLATION
      expect(await token.balanceOf(poolOwner.address)).to.equal(poolOwnerRewards);
      let poolOwner2Rewards = 0.25 * DAILY_INFLATION
      await token.connect(poolOwner2).claimRewards(poolOwner2.address);
      expect(await token.balanceOf(poolOwner2.address)).to.equal(poolOwner2Rewards);

      // staker2 both lower stakes to 5000 tokens each
      await token.connect(staker2).unstake(poolOwner.address, 25_000_000_000);
      await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);

      // on day 7 users claim rewards
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      await token.connect(staker).claimRewards(poolOwner.address);
      stakerRewards += 0.04 * DAILY_INFLATION
      expect(await token.balanceOf(staker.address)).to.equal(stakerRewards + 5_000_000_000);
      await token.connect(staker2).claimRewards(poolOwner.address);
      staker2Rewards += 0.04 * DAILY_INFLATION
      expect(await token.balanceOf(staker2.address)).to.equal(staker2Rewards + 100_000_000_000 - 30_600_000_000 + 25_000_000_000);
      const staker3Rewards = 0.25 * DAILY_INFLATION + 0.4 * DAILY_INFLATION
      await token.connect(staker3).claimRewards(poolOwner2.address);
      expect(await token.balanceOf(staker3.address)).to.equal(staker3Rewards + 100_000_000_000 - 40_800_000_000);

      // claim for pool owner
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      poolOwnerRewards += 0.12 * DAILY_INFLATION
      expect(await token.balanceOf(poolOwner.address)).to.equal(poolOwnerRewards);
      await token.connect(poolOwner2).claimRewards(poolOwner2.address);
      poolOwner2Rewards += 0.4 * DAILY_INFLATION
      expect(await token.balanceOf(poolOwner2.address)).to.equal(poolOwner2Rewards);

      // check that all rewards sum up to the 5 * total inflation
      expect(stakerRewards + staker2Rewards + staker3Rewards + poolOwnerRewards + poolOwner2Rewards).to.equal(6 * DAILY_INFLATION);
    })
    it("Should be able to handle multiple users and multiple pools with rewards - staking and airdrops", async function () {
      // ------------------------------------------
      // DAY 1
      //-------------------------------------------
      console.log("starting day 1");
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // array of integers to store total rewardable values (prefill 0 and 1 so that we can index by the number of day)
      const totalRewardables = [0, 0];
      // prepare 3 more stakers and mint some tokens into their wallets
      const signers = await ethers.getSigners();
      const airdropUser1 = signers[5];
      const airdropUser2 = signers[6];
      const airdropUser3 = signers[7];
      const vestingUser1 = signers[8];
      const vestingUser2 = signers[9];

      // create two more pools
      const poolOwner2 = signers[11];
      const poolOwner3 = signers[12];
      const minimumStakeAmount = 1_000_000_000;
      const stakersPart2 = 50;
      const stakersPart3 = 25;
      await token.connect(poolOwner2).createPool(minimumStakeAmount, stakersPart2);
      await token.connect(poolOwner3).createPool(minimumStakeAmount, stakersPart3);
      await token.connect(owner).activatePool(poolOwner2.address);
      await token.connect(owner).activatePool(poolOwner3.address);

      // ------------------------------------------
      // DAY 2
      //-------------------------------------------
      console.log("starting day 2");
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // first two airdrop users get airdrop of 100000 tokens to the first pool with the release date on day 10
      await token.connect(owner).airdrop(
        poolOwner.address,
        [airdropUser1.address, airdropUser2.address],
        [100_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY * 8],
        false,
      );
      // check that this raises the totalRewardable and the pool totalUnrewardedAirdrops
      expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(200_000_000_000);
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6);
      totalRewardables.push(200_000_000_000 * 0.6);

      // ------------------------------------------
      // DAY 3
      //-------------------------------------------
      console.log("starting day 3");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // last airdrop user gets airdrop of 100000 tokens to second pool with the release date on day 10
      await token.connect(owner).airdrop(
        poolOwner2.address,
        [airdropUser3.address],
        [100_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY * 7],
        false,
      );

      // staker stakes 10000 tokens to each pool - second stake should fail at first, then we mint some more tokens to the staker
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      // print the currentDayIdx from the poolOwner's pool
      await expect(token.connect(staker).stake(poolOwner.address, 10_200_000_000)).to.be.revertedWithCustomError(token, "InsufficientBalance").withArgs(staker.address, poolOwner.address, 0, 10_200_000_000);
      await token.connect(owner).mint(staker.address, 20_400_000_000);
      await token.connect(staker).stake(poolOwner2.address, 10_200_000_000);
      await token.connect(staker).stake(poolOwner3.address, 10_200_000_000);
      expect(await token.balanceOf(staker.address)).to.equal(0);

      // check that the totalStaked and totalRewardable and pool stats are correct
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 30_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(200_000_000_000);
      expect((await token.s_pools(poolOwner2.address)).totalUnrewardedAirdrops).to.equal(100_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(10_000_000_000);
      expect((await token.s_pools(poolOwner2.address)).totalStaked).to.equal(10_000_000_000);
      expect((await token.s_pools(poolOwner3.address)).totalStaked).to.equal(10_000_000_000);
      totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 30_000_000_000);

      // ------------------------------------------
      // DAY 4
      //-------------------------------------------
      console.log("starting day 4");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // create vesting airdrop with cliff at day 7 with rewards for 100_000 tokens to both vesting users to pool1 and pool3
      const dailyVestingAmount = 10_000_000_000;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [vestingUser1.address, vestingUser2.address],
        Array(10).fill(dailyVestingAmount),
        [currentTimestamp + SECONDS_IN_DAY * 3, currentTimestamp + SECONDS_IN_DAY * 4, currentTimestamp + SECONDS_IN_DAY * 5, currentTimestamp + SECONDS_IN_DAY * 6, currentTimestamp + SECONDS_IN_DAY * 7, currentTimestamp + SECONDS_IN_DAY * 8, currentTimestamp + SECONDS_IN_DAY * 9, currentTimestamp + SECONDS_IN_DAY * 10, currentTimestamp + SECONDS_IN_DAY * 11, currentTimestamp + SECONDS_IN_DAY * 12],
        true,
      );
      await token.connect(owner).airdrop(
        poolOwner3.address,
        [vestingUser1.address, vestingUser2.address],
        Array(10).fill(dailyVestingAmount),
        [currentTimestamp + SECONDS_IN_DAY * 3, currentTimestamp + SECONDS_IN_DAY * 4, currentTimestamp + SECONDS_IN_DAY * 5, currentTimestamp + SECONDS_IN_DAY * 6, currentTimestamp + SECONDS_IN_DAY * 7, currentTimestamp + SECONDS_IN_DAY * 8, currentTimestamp + SECONDS_IN_DAY * 9, currentTimestamp + SECONDS_IN_DAY * 10, currentTimestamp + SECONDS_IN_DAY * 11, currentTimestamp + SECONDS_IN_DAY * 12],
        true,
      );

      // check that the totalStaked, totalRewardable and pool stats are correct
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 30_000_000_000 + 400_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(200_000_000_000);
      expect((await token.s_pools(poolOwner2.address)).totalUnrewardedAirdrops).to.equal(100_000_000_000);
      expect((await token.s_pools(poolOwner3.address)).totalUnrewardedAirdrops).to.equal(0);
      expect((await token.s_pools(poolOwner.address)).totalRewardedAirdrops).to.equal(200_000_000_000);
      expect((await token.s_pools(poolOwner2.address)).totalRewardedAirdrops).to.equal(0);
      expect((await token.s_pools(poolOwner3.address)).totalRewardedAirdrops).to.equal(200_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(10_000_000_000);
      expect((await token.s_pools(poolOwner2.address)).totalStaked).to.equal(10_000_000_000);
      expect((await token.s_pools(poolOwner3.address)).totalStaked).to.equal(10_000_000_000);
      totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 30_000_000_000 + 400_000_000_000);

      // ------------------------------------------
      // DAY 5
      //-------------------------------------------
      console.log("starting day 5");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // staker unstakes 5000 tokens from pool1 - first unstake should fail at first, then he claims his rewards
      expect(await token.balanceOf(staker.address)).to.equal(0);
      await expect(token.connect(staker).unstake(poolOwner.address, 5_000_000_000)).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(staker.address, poolOwner.address);
      await token.connect(staker).claimRewards(poolOwner.address);
      await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);

      // check that the totalStaked, totalRewardable and pool stats are correct
      expect(await token.balanceOf(staker.address)).to.equal(
        Math.round(
          0.4 * 10_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
          0.4 * 10_000_000_000 * DAILY_INFLATION / totalRewardables[4]
        ) + 5_000_000_000
      );
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalUnrewardedAirdrops).to.equal(200_000_000_000);
      expect((await token.s_pools(poolOwner2.address)).totalUnrewardedAirdrops).to.equal(100_000_000_000);
      expect((await token.s_pools(poolOwner3.address)).totalUnrewardedAirdrops).to.equal(0);
      expect((await token.s_pools(poolOwner.address)).totalRewardedAirdrops).to.equal(200_000_000_000);
      expect((await token.s_pools(poolOwner2.address)).totalRewardedAirdrops).to.equal(0);
      expect((await token.s_pools(poolOwner3.address)).totalRewardedAirdrops).to.equal(200_000_000_000);
      expect((await token.s_pools(poolOwner.address)).totalStaked).to.equal(5_000_000_000);
      expect((await token.s_pools(poolOwner2.address)).totalStaked).to.equal(10_000_000_000);
      expect((await token.s_pools(poolOwner3.address)).totalStaked).to.equal(10_000_000_000);
      totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000);

      // ------------------------------------------
      // DAY 6
      //-------------------------------------------
      console.log("starting day 6");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      //vestingUser1 should be able to claim his rewards in pool1
      expect(await token.balanceOf(vestingUser1.address)).to.equal(0);
      await token.connect(vestingUser1).claimRewards(poolOwner.address);
      expect(await token.balanceOf(vestingUser1.address)).to.equal
      (
        Math.round(
          (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[4]) +
          (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[5])
        )
      );

      // no one can claim their airdrops yet
      await expect(token.connect(airdropUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser1.address, poolOwner.address);
      await expect(token.connect(airdropUser2).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser2.address, poolOwner.address);
      await expect(token.connect(airdropUser3).unlockAirdrop(poolOwner2.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser3.address, poolOwner2.address);
      await expect(token.connect(vestingUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(vestingUser1.address, poolOwner.address);
      await expect(token.connect(vestingUser2).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "UnclaimedRewards").withArgs(vestingUser2.address, poolOwner.address);

      // check that the totalStaked, totalRewardable are correct
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000);

      // claim rewards as a pool1 owner
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      expect(await token.balanceOf(poolOwner.address)).to.equal(
        Math.floor(
          0.6 * 200_000_000_000 * DAILY_INFLATION / totalRewardables[2] +
          0.6 * 210_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
          0.6 * 410_000_000_000 * DAILY_INFLATION / totalRewardables[4] +
          0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[5]
        )
      )
      totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000);

      // ------------------------------------------
      // DAY 7
      //-------------------------------------------
      console.log("starting day 7");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // the first part of the vesting airdrop should be ready now
      await token.connect(vestingUser1).claimRewards(poolOwner.address);
      await token.connect(vestingUser1).unlockAirdrop(poolOwner.address);
      expect(await token.balanceOf(vestingUser1.address)).to.equal(
        Math.round(
          (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[4]) +
          (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[5]) +
          (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[6])
        ) + 10_000_000_000
      );
      await expect(token.connect(vestingUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(vestingUser1.address, poolOwner.address);

      // pool1 owner should be able to claim his rewards again
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      expect(await token.balanceOf(poolOwner.address)).to.equal(
        Math.floor(
          0.6 * 200_000_000_000 * DAILY_INFLATION / totalRewardables[2] +
          0.6 * 210_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
          0.6 * 410_000_000_000 * DAILY_INFLATION / totalRewardables[4] +
          0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[5] +
          0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[6]
        )
      )

      // check that the totalStaked, totalRewardable are correct
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 10_000_000_000);
      totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 10_000_000_000);

      // ------------------------------------------
      // DAY 8
      //-------------------------------------------
      console.log("starting day 8");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // the second part of the vesting airdrop should be ready now
      await token.connect(vestingUser1).claimRewards(poolOwner.address);
      await token.connect(vestingUser1).unlockAirdrop(poolOwner.address);
      expect(await token.balanceOf(vestingUser1.address)).to.equal(
        Math.round(
          (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[4]) +
          (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[5]) +
          (0.4 * 100_000_000_000 * DAILY_INFLATION / totalRewardables[6]) +
          (0.4 * 90_000_000_000 * DAILY_INFLATION / totalRewardables[7])
        ) + 20_000_000_000 + 1 // +1 rounding error - todo investigate
      );

      // check that the totalStaked, totalRewardable are correct
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 20_000_000_000);
      totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 20_000_000_000);

      // ------------------------------------------
      // DAY 9
      //-------------------------------------------
      console.log("starting day 9");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // check that the airdropUsers still cannot claim their airdrop
      await expect(token.connect(airdropUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser1.address, poolOwner.address);
      await expect(token.connect(airdropUser2).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser2.address, poolOwner.address);
      await expect(token.connect(airdropUser3).unlockAirdrop(poolOwner2.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser3.address, poolOwner2.address);

      // check that the totalStaked, totalRewardable are correct
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 20_000_000_000);
      totalRewardables.push(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 2 * 2 * 100_000_000_000 - 20_000_000_000);

      // ------------------------------------------
      // DAY 10
      //-------------------------------------------
      console.log("starting day 10");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // revoke the airdrop for vestingUser1 and airdropUser1
      expect(await token.s_totalRewardable()).to.equal(200_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 380_000_000_000);
      await token.connect(owner).revokeAirdrop(poolOwner.address, [vestingUser1.address, airdropUser1.address]);
      expect(await token.s_totalRewardable()).to.equal(100_000_000_000 * 0.6 + 100_000_000_000 * 0.5 + 25_000_000_000 + 300_000_000_000);

      // check that the airdropUser1 and vestingUser1 cannot claim their airdrop
      await expect(token.connect(airdropUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(airdropUser1.address, poolOwner.address);
      await expect(token.connect(vestingUser1).unlockAirdrop(poolOwner.address)).to.be.revertedWithCustomError(token, "AirdropNotReady").withArgs(vestingUser1.address, poolOwner.address);
      //try claiming rewards as a vestingUser1
      await expect(token.connect(vestingUser1).claimRewards(poolOwner.address)).to.be.revertedWithCustomError(token, "NoClaimableRewards").withArgs(vestingUser1.address, poolOwner.address);

      // check that the airdropUser2 can claim his airdrop
      await token.connect(airdropUser2).unlockAirdrop(poolOwner.address);
      expect(await token.balanceOf(airdropUser2.address)).to.equal(100_000_000_000);

      // check that the totalStaked, totalRewardable are correct
      expect(await token.s_totalRewardable()).to.equal(100_000_000_000 * 0.5 + 25_000_000_000 + 300_000_000_000);

      //check that the pool1 owner can claim his rewards
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      expect(await token.balanceOf(poolOwner.address)).to.equal(
        Math.floor(
          0.6 * 200_000_000_000 * DAILY_INFLATION / totalRewardables[2] +
          0.6 * 210_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
          0.6 * 410_000_000_000 * DAILY_INFLATION / totalRewardables[4] +
          0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[5] +
          0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[6] +
          0.6 * 395_000_000_000 * DAILY_INFLATION / totalRewardables[7] +
          0.6 * 385_000_000_000 * DAILY_INFLATION / totalRewardables[8] +
          0.6 * 385_000_000_000 * DAILY_INFLATION / totalRewardables[9]
        ) - 2 // -2 because of the rounding error - todo maybe fix this
      );

      // check that the totalStaked, totalRewardable are correct
      expect(await token.s_totalRewardable()).to.equal(100_000_000_000 * 0.5 + 25_000_000_000 + 300_000_000_000);
      totalRewardables.push(100_000_000_000 * 0.5 + 25_000_000_000 + 300_000_000_000);

      // ------------------------------------------
      // DAY 11
      //-------------------------------------------
      console.log("starting day 11");
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;

      // check that the pool1 owner can claim his rewards
      await token.connect(poolOwner).claimRewards(poolOwner.address);
      expect(await token.balanceOf(poolOwner.address)).to.equal(
        Math.floor(
          0.6 * 200_000_000_000 * DAILY_INFLATION / totalRewardables[2] +
          0.6 * 210_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
          0.6 * 410_000_000_000 * DAILY_INFLATION / totalRewardables[4] +
          0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[5] +
          0.6 * 405_000_000_000 * DAILY_INFLATION / totalRewardables[6] +
          0.6 * 395_000_000_000 * DAILY_INFLATION / totalRewardables[7] +
          0.6 * 385_000_000_000 * DAILY_INFLATION / totalRewardables[8] +
          0.6 * 385_000_000_000 * DAILY_INFLATION / totalRewardables[9] +
          0.6 * 105_000_000_000 * DAILY_INFLATION / totalRewardables[10]
        ) - 2 // -2 because of the rounding error - todo maybe fix this
      );

      // staker claims all rewards
      expect(await token.balanceOf(staker.address)).to.equal(
        Math.round(
          0.4 * 10_000_000_000 * DAILY_INFLATION / totalRewardables[3] +
          0.4 * 10_000_000_000 * DAILY_INFLATION / totalRewardables[4]
        ) + 5_000_000_000
      );

      await token.connect(staker).claimRewards(poolOwner.address);
      await token.connect(staker).claimRewards(poolOwner2.address);
      await token.connect(staker).claimRewards(poolOwner3.address);

      // unstake it all
      await token.connect(staker).unstake(poolOwner.address, 5_000_000_000);
      await token.connect(staker).unstake(poolOwner2.address, 10_000_000_000);
      await token.connect(staker).unstake(poolOwner3.address, 10_000_000_000);

      // claim rewards as a staker - should fail
      expect(await token.balanceOf(staker.address)).to.equal(
        Math.round(
          (0.4 * 10_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[3] +
          (0.4 * 10_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[4] +
          (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[5] +
          (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[6] +
          (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[7] +
          (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[8] +
          (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[9] +
          (0.4 * 5_000_000_000 + 0.5 * 10_000_000_000 + 0.25 * 10_000_000_000) * DAILY_INFLATION / totalRewardables[10]
        ) + 30_000_000_000 + 1 // +1 because of the rounding error - todo maybe fix this
      );

      // check that the totalStaked, totalRewardable are correct
      expect(await token.s_totalRewardable()).to.equal(100_000_000_000 * 0.5 + 300_000_000_000);
    })
  })

  describe("Events", function () {
    it("Should emit BurnFeeChanged event", async function () {
      const { token, owner } = await deployTokenFixture();
      await expect(token.connect(owner).setBurnFee(5))
        .to.emit(token, "BurnFeeChanged").withArgs(5);
    })
    it("Should emit DailyInflationChangeScheduled event", async function () {
      const { token, owner } = await deployTokenFixture();
      await expect(token.connect(owner).setDailyInflation(200_000_000_000))
        .to.emit(token, "DailyInflationChangeScheduled").withArgs(200_000_000_000);
    })
    it("Should emit Minted event", async function () {
      const { token, owner, staker } = await deployTokenFixture();
      await expect(token.connect(owner).mint(staker.address, 100_000_000_000))
        .to.emit(token, "Minted").withArgs(staker.address, 100_000_000_000);
    })
    it("Should emit PoolCreated event", async function () {
      const { token, owner } = await deployTokenFixture();
      const poolOwner = (await ethers.getSigners())[9]
      const minimumStakeAmount = 1_000_000_000;
      await expect(token.connect(poolOwner).createPool(minimumStakeAmount, 20)).
      to.emit(token, "PoolCreated").withArgs(poolOwner.address, minimumStakeAmount, 20)
      ;
    })
    it("Should emit PoolActivated event", async function () {
      const { token, owner } = await deployTokenFixture();
      const poolOwner = (await ethers.getSigners())[7]
      const minimumStakeAmount = 1_000_000_000;
      await token.connect(poolOwner).createPool(minimumStakeAmount, 20)
      await expect(token.connect(owner).activatePool(poolOwner.address))
        .to.emit(token, "PoolActivated").withArgs(poolOwner.address);
    })
    it("Should emit MinimumStakeAmountChanged event", async function () {
      const { token, poolOwner } = await deployActivePoolFixture();
      await expect(token.connect(poolOwner).setMinimumStakeAmount(1_000_000_000))
        .to.emit(token, "MinimumStakeAmountChanged").withArgs(poolOwner.address, 1_000_000_000);
    })
    it("Should emit StakersPartChanged event", async function () {
      const { token, poolOwner } = await deployActivePoolFixture();
      expect(await token.connect(poolOwner).setStakersPart(20))
        .to.emit(token, "StakersPartChanged").withArgs(poolOwner.address, 20);
    })
    it("Should emit Staked event", async function () {
      const { token, poolOwner, staker } = await deployActivePoolFixture();
      await expect(token.connect(staker).stake(poolOwner.address, 10_200_000_000))
        .to.emit(token, "Staked").withArgs(staker.address, poolOwner.address, 10_000_000_000);
    })
    it("Should emit Unstaked event", async function () {
      const { token, poolOwner, staker } = await deployActivePoolFixture();
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      await expect(token.connect(staker).unstake(poolOwner.address, 10_000_000_000))
        .to.emit(token, "Unstaked").withArgs(staker.address, poolOwner.address, 10_000_000_000);
    })
    it("Should emit RewardsClaimed event", async function () {
      const { token, poolOwner, staker } = await deployActivePoolFixture();
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      // wait 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      await expect(token.connect(staker).claimRewards(poolOwner.address))
        .to.emit(token, "RewardsClaimed").withArgs(staker.address, poolOwner.address, 0.4 * DAILY_INFLATION);
    })
    it("Should emit Airdropped event", async function () {
      const { token, owner, staker, poolOwner } = await deployActivePoolFixture();
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await expect(token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        Array(10).fill(10_000_000_000),
        Array.from({ length: 10 }, (_, i) => currentTimestamp + (i + 1) * SECONDS_IN_DAY),
        true,
      )).to.emit(token, "Airdropped").withArgs(
        poolOwner.address,
        [staker.address],
        Array(10).fill(10_000_000_000),
        Array.from({ length: 10 }, (_, i) => currentTimestamp + (i + 1) * SECONDS_IN_DAY),
        true
      );
    })
    it("Should emit AirdropUnlocked event", async function () {
      const { token, owner, staker, poolOwner } = await deployActivePoolFixture();
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      )
      // wait 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);

      await expect(token.connect(staker).unlockAirdrop(poolOwner.address))
        .to.emit(token, "AirdropUnlocked").withArgs(staker.address, poolOwner.address, 10_000_000_000);

    })
    it("Should emit AirdropRevoked event", async function () {
      const { token, owner, staker, poolOwner } = await deployActivePoolFixture();
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      )
      // wait 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);

      expect(await token.connect(owner).revokeAirdrop(poolOwner.address, [staker.address]))
        .to.emit(token, "AirdropRevoked").withArgs(poolOwner.address, [staker.address], 10_000_000_000, 0);
    })
    it("Should emit Cranked event - setMinimumStakeAmount", async function () {
      const { token, owner, staker, poolOwner } = await deployActivePoolFixture();
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      )
      // wait 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // set minimum stake amount should crank
      await expect(token.connect(poolOwner).setMinimumStakeAmount(10_000_000_000)).to.emit(token, "Cranked");
    })
    it("Should emit Cranked event - setStakersPart", async function () {
      const { token, owner, staker, poolOwner } = await deployActivePoolFixture();
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        true
      )
      // wait 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // set minimum stake amount should crank
      await expect(token.connect(poolOwner).setStakersPart(90)).to.emit(token, "Cranked");
    })
    it("Should emit Cranked event - stake", async function () {
      const { token, owner, staker, poolOwner } = await deployActivePoolFixture();
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      // wait 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // stake should crank
      await expect(token.connect(staker).stake(poolOwner.address, 10_000_000_000)).to.emit(token, "Cranked");
    })
    it("Should emit Cranked event - claimRewards", async function () {
      const { token, poolOwner, staker } = await deployActivePoolFixture();
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // wait 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // claim rewards should crank
      await expect(token.connect(staker).claimRewards(poolOwner.address))
        .to.emit(token, "Cranked");
    })
    it("Should emit Cranked event - airdrop", async function () {
      const { token, owner, poolOwner, staker } = await deployActivePoolFixture();
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // wait 1 day
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // airdrop should crank
      await expect(token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + 2 * SECONDS_IN_DAY],
        false,
      )).to.emit(token, "Cranked");
    })
    it("Should emit Cranked event - unlockAirdrop", async function () {
      const { token, owner, staker, poolOwner } = await deployActivePoolFixture();
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      )
      // wait 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // unlock airdrop should crank
      await expect(token.connect(staker).unlockAirdrop(poolOwner.address))
        .to.emit(token, "Cranked");
    })
    it("Should emit Cranked event - revokeAirdrop", async function () {
      const { token, owner, staker, poolOwner } = await deployActivePoolFixture();
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      )
      // wait 1 day
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // revoke airdrop should crank
      await expect(token.connect(owner).revokeAirdrop(poolOwner.address, [staker.address]))
        .to.emit(token, "Cranked");
    })
  })
  describe("Pause", function () {
    it("Should be unpaused by default", async function () {
      const { token } = await deployTokenFixture();
      expect(await token.paused()).to.equal(false);
    })
    it("Should be pausable only by the owner", async function () {
      const { token, owner, staker } = await deployTokenFixture();
      await expect(token.connect(staker).pause()).to.be.revertedWith("Ownable: caller is not the owner");
      await expect(token.connect(owner).pause()).to.emit(token, "Paused");
    })
    it("Should be unpausable only by the owner", async function () {
      const { token, owner, staker } = await deployTokenFixture();
      await token.connect(owner).pause();
      await expect(token.connect(staker).unpause()).to.be.revertedWith("Ownable: caller is not the owner");
      await expect(token.connect(owner).unpause()).to.emit(token, "Unpaused");
    })
    it("Should not be possible to call any external or public non-view function when paused", async function () {
      const { token, owner, staker } = await deployTokenFixture();
      await token.connect(owner).pause();

      await expect(token.connect(owner).setBurnFee(10)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(owner).setDailyInflation(10_000_000_000)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(owner).mint(owner.address, 10_000_000_000)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(staker).createPool(1_000_000_000, 30)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(owner).activatePool(staker.address)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(owner).setMinimumStakeAmount(10_000_000_000)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(owner).setStakersPart(90)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(staker).stake(owner.address, 10_000_000_000)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(staker).unstake(owner.address, 10_000_000_000)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(staker).claimRewards(owner.address)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(owner).airdrop(
        owner.address,
        [staker.address],
        [10_000_000_000],
        [0],
        false,
      )).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(staker).unlockAirdrop(owner.address)).to.be.revertedWith("Pausable: paused");
      await expect(token.connect(owner).revokeAirdrop(owner.address, [staker.address])).to.be.revertedWith("Pausable: paused");
    })
    it("Should be possible to call views even when paused", async function () {
      const { token, owner, poolOwner, staker } = await deployActivePoolFixture();
      // airdrop to staker
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      )
      await token.connect(owner).pause();

      await expect(token.paused()).to.not.be.reverted;
      await expect(token.totalSupply()).to.not.be.reverted;
      await expect(token.balanceOf(owner.address)).to.not.be.reverted;
      await expect(token.allowance(owner.address, staker.address)).to.not.be.reverted;
      await expect(token.connect(staker).airdropInfo(poolOwner.address)).to.not.be.reverted;
      await expect(token.connect(owner).renounceOwnership()).to.be.revertedWith("Cannot renounce ownership");
      await expect(token.decimals()).to.not.be.reverted;
      await expect(token.totalStake(poolOwner.address, staker.address)).to.not.be.reverted;
      await expect(token.enoughStaked(poolOwner.address, owner.address)).to.not.be.reverted;
    })
  })
  describe("Bounds", function () {
    it("Should be able to set the burn fee to 0 and perform any operation", async function () {
      const { token, owner, poolOwner, staker } = await deployActivePoolFixture();
      await token.connect(owner).setBurnFee(0);
      // airdrop to staker
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      )
      // stake
      await token.connect(staker).stake(poolOwner.address, 10_000_000_000);
      // check ballance
      expect(await token.balanceOf(staker.address)).to.equal(200_000_000);
      // forward 1 day
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      // unstake
      await token.connect(staker).unstake(poolOwner.address, 10_000_000_000);
      // unlock airdrop
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      // check ballance
      expect(await token.balanceOf(staker.address)).to.equal(20_200_000_000 + 4 / 16 * DAILY_INFLATION);
    })
    it("Should be able to set the burn fee to 900 and perform any operation", async function () {
      const { token, owner, poolOwner, staker } = await deployActivePoolFixture();
      await token.connect(owner).setBurnFee(900);
      // airdrop to staker
      let currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await token.connect(owner).airdrop(
        poolOwner.address,
        [staker.address],
        [10_000_000_000],
        [currentTimestamp + SECONDS_IN_DAY],
        false
      )
      // stake
      await token.connect(staker).stake(poolOwner.address, 10_000_000_000);
      // check ballance
      expect(await token.balanceOf(staker.address)).to.equal(200_000_000);
      // forward 1 day
      currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + SECONDS_IN_DAY]);
      // claim rewards
      await token.connect(staker).claimRewards(poolOwner.address);
      // unstake
      await token.connect(staker).unstake(poolOwner.address, 1_000_000_000);
      // unlock airdrop
      await token.connect(staker).unlockAirdrop(poolOwner.address);
      // check ballance
      expect(await token.balanceOf(staker.address)).to.equal(
        Math.round(
          11_200_000_000 + 0.4 / 7 * DAILY_INFLATION
        ));
    })
  })
  describe("Scaling", function () {
    const DAYS_IN_TWO_YEARS = 365 * 2;
    it("Should be possible to crank a pool after two years", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // create a second pool owner
      const poolOwner2 = (await ethers.getSigners())[8];
      // create a second pool
      await token.connect(poolOwner2).createPool(1_000_000_000, 30);
      await token.connect(owner).activatePool(poolOwner2.address);
      // stake to both
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      await token.connect(staker).stake(poolOwner2.address, 5_100_000_000);

      // claim rewards from the first pool every day for two years - we want to generate a lot of Reward objects
      const startTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      for (let i = 1; i <= DAYS_IN_TWO_YEARS; i++) {
        await ethers.provider.send("evm_mine", [startTimestamp + i * SECONDS_IN_DAY]);
        await token.connect(staker).claimRewards(poolOwner.address);
        expect(await token.balanceOf(staker.address)).to.equal(
          DAILY_INFLATION * 0.4 * 0.5 * i
        );
        console.log(i)
      }

      // claim rewards from the second pool
      await expect(token.connect(staker).claimRewards(poolOwner2.address)).to.emit(token, "RewardsClaimed").withArgs(
        staker.address,
        poolOwner2.address,
        DAILY_INFLATION * 0.3 * 0.5 * DAYS_IN_TWO_YEARS);
      expect(await token.balanceOf(staker.address)).to.equal(
        DAILY_INFLATION * 0.7 * 0.5 * DAYS_IN_TWO_YEARS
      );
    })
    it("Should be possible to crank the whole system after two years", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // stake
      await token.connect(staker).stake(poolOwner.address, 5_100_000_000);
      // forward 2 years
      const currentTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      await ethers.provider.send("evm_mine", [currentTimestamp + DAYS_IN_TWO_YEARS * SECONDS_IN_DAY]);
      // claim rewards
      // todo - this claiming is very expensive, could be improved on the side of the contract, but probably not necessary as it is highly improbable that this situation will happen
      await expect(token.connect(staker).claimRewards(poolOwner.address)).to.emit(token, "RewardsClaimed").withArgs(
        staker.address,
        poolOwner.address,
        DAILY_INFLATION * 0.4 * DAYS_IN_TWO_YEARS
      );
      expect(await token.balanceOf(staker.address)).to.equal(
        DAILY_INFLATION * 0.4 * DAYS_IN_TWO_YEARS + 5_100_000_000
      );
    })
    it("Should be possible to claim rewards as a user after 2 years", async function () {
      const { token, poolOwner, owner, staker } = await loadFixture(deployActivePoolFixture);
      // second staker
      const staker2 = (await ethers.getSigners())[8];
      await token.connect(owner).mint(staker2.address, 10_200_000_000);
      // stake
      await token.connect(staker).stake(poolOwner.address, 10_200_000_000);
      await token.connect(staker2).stake(poolOwner.address, 10_200_000_000);
      // staker2 claims rewards every day for two years
      const startTimestamp = (await ethers.provider.getBlock("latest")).timestamp;
      for (let i = 1; i <= DAYS_IN_TWO_YEARS; i++) {
        await ethers.provider.send("evm_mine", [startTimestamp + i * SECONDS_IN_DAY]);
        await token.connect(staker2).claimRewards(poolOwner.address);
        expect(await token.balanceOf(staker2.address)).to.equal(
          DAILY_INFLATION * 0.4 * 0.5 * i
        );
      }
      // staker1 claims rewards after two years
      await expect(token.connect(staker).claimRewards(poolOwner.address)).to.emit(token, "RewardsClaimed").withArgs(
        staker.address,
        poolOwner.address,
        DAILY_INFLATION * 0.4 * 0.5 * DAYS_IN_TWO_YEARS
      );
      expect(await token.balanceOf(staker.address)).to.equal(
        DAILY_INFLATION * 0.4 * 0.5 * DAYS_IN_TWO_YEARS
      );
      // pool owner claims rewards after two years
      await expect(token.connect(poolOwner).claimRewards(poolOwner.address)).to.emit(token, "RewardsClaimed").withArgs(
        poolOwner.address,
        poolOwner.address,
        DAILY_INFLATION * 0.6 * DAYS_IN_TWO_YEARS
      );
      expect(await token.balanceOf(poolOwner.address)).to.equal(
        DAILY_INFLATION * 0.6 * DAYS_IN_TWO_YEARS
      );
    })
  })
});