import { afterAll, beforeAll, expect, jest, test } from "@jest/globals";
import { ChildProcess } from "child_process";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  airdropPayer,
  deployProgram,
  initializePayer,
  spawnLocalSolana,
  signAndSendTransactionInstructions,
} from "./utils";
import {
  createCentralState,
  createStakeAccount,
  createStakePool,
  stake,
  createBond,
  claimBond,
  unlockBondTokens,
  claimBondRewards,
  crank,
  claimPoolRewards,
  claimRewards,
  changeInflation,
  changePoolMinimum,
  unstake,
  adminMint,
  activateStakePool,
  adminFreeze,
  changePoolMultiplier,
} from "../src/bindings";
import {
  CentralState,
  Tag,
  StakePool,
  StakeAccount,
  BondAccount,
  UnstakeRequest,
} from "../src/state";
import {
  Token,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { sleep } from "../src/utils";
import BN from "bn.js";
import { TokenMint } from "./utils";

// Global state initialized once in test startup and cleaned up at test
// teardown.
let solana: ChildProcess;
let connection: Connection;
let feePayer: Keypair;
let payerKeyFile: string;
let programId: PublicKey;
const delay = 30_000;
const MAX_i64 = "9223372036854775807";

beforeAll(async () => {
  solana = await spawnLocalSolana();
  connection = new Connection("http://localhost:8899", "finalized");
  [feePayer, payerKeyFile] = initializePayer();
  await airdropPayer(connection, feePayer.publicKey);
  programId = deployProgram(
    payerKeyFile,
    true,
    "no-lock-time no-mint-check no-bond-signer",
    true
  );
});

afterAll(() => {
  if (solana !== undefined) {
    try {
      solana.kill();
    } catch (e) {
      console.log(e);
    }
  }
});

jest.setTimeout(1_500_000);

test("End to end test", async () => {
  /**
   * Test variables
   */
  const [centralKey, centralNonce] = await CentralState.getKey(programId);
  const decimals = Math.pow(10, 6);
  let dailyInflation = 1_000_000 * decimals;
  const centralStateAuthority = Keypair.generate();
  const accessToken = await TokenMint.init(connection, feePayer, centralKey);
  const quoteToken = await TokenMint.init(connection, feePayer);
  const stakePoolOwner = Keypair.generate();
  const staker = Keypair.generate();
  let minimumStakeAmount = 10_000 * decimals;
  const bondAmount = 5_000_000 * decimals;
  const bondSeller = Keypair.generate();
  let fees = 0; // Fees collected by the central state
  let FEES = 5 / 100; // % of fees collected on each stake

  await airdropPayer(connection, bondSeller.publicKey);

  /**
   * Set up ATA
   */

  const stakePoolAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    accessToken.token.publicKey,
    stakePoolOwner.publicKey
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      accessToken.token.publicKey,
      stakePoolAta,
      stakePoolOwner.publicKey,
      feePayer.publicKey
    ),
  ]);

  const stakerAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    accessToken.token.publicKey,
    staker.publicKey
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      accessToken.token.publicKey,
      stakerAta,
      staker.publicKey,
      feePayer.publicKey
    ),
  ]);

  const feesAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    accessToken.token.publicKey,
    centralStateAuthority.publicKey
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      accessToken.token.publicKey,
      feesAta,
      centralStateAuthority.publicKey,
      feePayer.publicKey
    ),
  ]);

  /**
   * Create central state
   */

  const ix_central_state = await createCentralState(
    dailyInflation,
    centralStateAuthority.publicKey,
    feePayer.publicKey,
    accessToken.token.publicKey,
    programId
  );

  let tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_central_state,
  ]);
  console.log(`Created centrale state ${tx}`);

  // Verifications

  let centralStateObj = await CentralState.retrieve(connection, centralKey);
  expect(centralStateObj.tag).toBe(Tag.CentralState);
  expect(centralStateObj.signerNonce).toBe(centralNonce);
  expect(centralStateObj.dailyInflation.toNumber()).toBe(dailyInflation);
  expect(centralStateObj.tokenMint.toBase58()).toBe(
    accessToken.token.publicKey.toBase58()
  );
  expect(centralStateObj.authority.toBase58()).toBe(
    centralStateAuthority.publicKey.toBase58()
  );

  /**
   * Create stake pool
   */
  const [stakePoolKey, stakePoolNonce] = await StakePool.getKey(
    programId,
    stakePoolOwner.publicKey
  );
  const vault = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    accessToken.token.publicKey,
    stakePoolKey,
    true
  );
  const ix_stake_pool = await createStakePool(
    connection,
    stakePoolOwner.publicKey,
    minimumStakeAmount,
    feePayer.publicKey,
    programId
  );

  tx = await signAndSendTransactionInstructions(
    connection,
    [],
    feePayer,
    ix_stake_pool
  );
  console.log(`Created stake pool ${tx}`);

  // Verifications
  let now = Math.floor(new Date().getTime() / 1_000);
  let stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.InactiveStakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBe(0);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(0);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeLessThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  /**
   * Activate stake pool
   */
  const ix_act_stake_pool = await activateStakePool(
    connection,
    stakePoolKey,
    programId
  );

  tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [ix_act_stake_pool]
  );

  //Verification
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);

  // Create stake account
  const [stakeKey, stakeNonce] = await StakeAccount.getKey(
    programId,
    staker.publicKey,
    stakePoolKey
  );
  const ix_create_stake_acc = await createStakeAccount(
    stakePoolKey,
    staker.publicKey,
    feePayer.publicKey,
    programId
  );
  tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_create_stake_acc,
  ]);

  /**
   * Verifications
   */

  now = Math.floor(new Date().getTime() / 1_000);
  const stakeAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  expect(stakeAccountObj.tag).toBe(Tag.StakeAccount);
  expect(stakeAccountObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(stakeAccountObj.stakeAmount.toNumber()).toBe(0);
  expect(stakeAccountObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(stakeAccountObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakeAccountObj.poolMinimumAtCreation.toNumber()).toBe(
    minimumStakeAmount
  );
  expect(stakeAccountObj.pendingUnstakeRequests).toBe(0);
  expect(JSON.stringify(stakeAccountObj.unstakeRequests)).toBe(
    JSON.stringify([
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
    ])
  );

  /**
   * Create a bond
   *
   */

  const quoteBuyerAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    quoteToken.token.publicKey,
    staker.publicKey
  );
  const ix_quote_buyer_ata = Token.createAssociatedTokenAccountInstruction(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    quoteToken.token.publicKey,
    quoteBuyerAta,
    staker.publicKey,
    feePayer.publicKey
  );
  const quoteSellerAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    quoteToken.token.publicKey,
    bondSeller.publicKey
  );
  const ix_quote_seller_ata = Token.createAssociatedTokenAccountInstruction(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    quoteToken.token.publicKey,
    quoteSellerAta,
    bondSeller.publicKey,
    feePayer.publicKey
  );

  tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_quote_buyer_ata,
    ix_quote_seller_ata,
  ]);

  await quoteToken.mintInto(quoteBuyerAta, bondAmount);

  const [bondKey] = await BondAccount.getKey(
    programId,
    staker.publicKey,
    bondAmount
  );
  const ix_create_bond = await createBond(
    bondSeller.publicKey,
    staker.publicKey,
    bondAmount,
    bondAmount,
    quoteToken.token.publicKey,
    quoteSellerAta,
    0,
    1,
    bondAmount,
    stakePoolKey,
    0,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [bondSeller],
    feePayer,
    [ix_create_bond]
  );

  // Verifications
  let bondObj = await BondAccount.retrieve(connection, bondKey);
  expect(bondObj.tag).toBe(Tag.InactiveBondAccount);
  expect(bondObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(bondObj.totalAmountSold.toNumber()).toBe(bondAmount);
  expect(bondObj.totalStaked.toNumber()).toBe(bondAmount);
  expect(bondObj.totalQuoteAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.quoteMint.toBase58()).toBe(
    quoteToken.token.publicKey.toBase58()
  );
  expect(bondObj.sellerTokenAccount.toBase58()).toBe(quoteSellerAta.toBase58());
  expect(bondObj.unlockStartDate.toNumber()).toBe(0);
  expect(bondObj.unlockPeriod.toNumber()).toBe(1);
  expect(bondObj.unlockAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.lastUnlockTime.toNumber()).toBe(0);
  expect(bondObj.totalUnlockedAmount.toNumber()).toBe(0);
  expect(bondObj.poolMinimumAtCreation.toNumber()).toBe(minimumStakeAmount);
  expect(bondObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(
    bondObj.lastClaimedTime.div(new BN(2 ** 11)).toNumber()
  ).toBeGreaterThan(now);
  expect(bondObj.sellers.length).toBe(1);
  expect(bondObj.sellers[0].toBase58()).toBe(bondSeller.publicKey.toBase58());

  /**
   * Claim bond
   */

  const ix_claim_bond = await claimBond(
    connection,
    bondKey,
    staker.publicKey,
    quoteBuyerAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_claim_bond]
  );

  // Verifications
  bondObj = await BondAccount.retrieve(connection, bondKey);
  expect(bondObj.tag).toBe(Tag.BondAccount);
  expect(bondObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(bondObj.totalAmountSold.toNumber()).toBe(bondAmount);
  expect(bondObj.totalStaked.toNumber()).toBe(bondAmount);
  expect(bondObj.totalQuoteAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.quoteMint.toBase58()).toBe(
    quoteToken.token.publicKey.toBase58()
  );
  expect(bondObj.sellerTokenAccount.toBase58()).toBe(quoteSellerAta.toBase58());
  expect(bondObj.unlockStartDate.toNumber()).toBe(0);
  expect(bondObj.unlockPeriod.toNumber()).toBe(1);
  expect(bondObj.unlockAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.lastUnlockTime.toNumber()).toBeGreaterThan(now);
  expect(bondObj.totalUnlockedAmount.toNumber()).toBe(0);
  expect(bondObj.poolMinimumAtCreation.toNumber()).toBe(minimumStakeAmount);
  expect(bondObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(bondObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(bondObj.sellers.length).toBe(1);
  expect(bondObj.sellers[0].toBase58()).toBe(bondSeller.publicKey.toBase58());

  /**
   * Unlock bond tokens
   */

  let preBalance = await (
    await connection.getTokenAccountBalance(stakerAta)
  ).value.amount;
  expect(preBalance).toBe("0");
  await sleep(15_000);
  const ix_unlock_bond_tokens = await unlockBondTokens(
    connection,
    bondKey,
    stakerAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_unlock_bond_tokens]
  );

  // Verifications
  now = Math.floor(new Date().getTime() / 1_000);
  bondObj = await BondAccount.retrieve(connection, bondKey);
  let postBalance = await (
    await connection.getTokenAccountBalance(stakerAta)
  ).value.amount;
  expect(postBalance).toBe(new BN(bondAmount).toString());

  expect(bondObj.tag).toBe(Tag.BondAccount);
  expect(bondObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(bondObj.totalAmountSold.toNumber()).toBe(bondAmount);
  expect(bondObj.totalStaked.toNumber()).toBe(0);
  expect(bondObj.totalQuoteAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.quoteMint.toBase58()).toBe(
    quoteToken.token.publicKey.toBase58()
  );
  expect(bondObj.sellerTokenAccount.toBase58()).toBe(quoteSellerAta.toBase58());
  expect(bondObj.unlockStartDate.toNumber()).toBe(0);
  expect(bondObj.unlockPeriod.toNumber()).toBe(1);
  expect(bondObj.unlockAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.lastUnlockTime.toNumber()).toBeLessThan(now);
  expect(bondObj.totalUnlockedAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.poolMinimumAtCreation.toNumber()).toBe(minimumStakeAmount);
  expect(bondObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(bondObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(bondObj.sellers.length).toBe(1);
  expect(bondObj.sellers[0].toBase58()).toBe(bondSeller.publicKey.toBase58());

  // Stake
  preBalance = await (
    await connection.getTokenAccountBalance(stakerAta)
  ).value.amount;

  let stakeAmount = 20_000 * decimals;

  let ix_stake = await stake(
    connection,
    stakeKey,
    stakerAta,
    stakeAmount,
    feesAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_stake]
  );

  /**
   * Verifications
   */

  now = Math.floor(new Date().getTime() / 1_000);
  await sleep(5_000);
  postBalance = (await connection.getTokenAccountBalance(stakerAta)).value
    .amount;
  expect(postBalance).toBe(
    new BN(new BN(preBalance).sub(new BN(stakeAmount))).toString()
  );

  fees = Math.floor(stakeAmount * FEES);
  stakeAmount -= fees;

  let stakedAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  expect(stakedAccountObj.tag).toBe(Tag.StakeAccount);
  expect(stakedAccountObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(stakedAccountObj.stakeAmount.toNumber()).toBe(stakeAmount);
  expect(stakedAccountObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(stakedAccountObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakedAccountObj.poolMinimumAtCreation.toNumber()).toBe(
    minimumStakeAmount
  );

  let feesTokenAcc = await connection.getTokenAccountBalance(feesAta);
  expect(parseInt(feesTokenAcc.value.amount)).toBe(fees);

  // Crank
  let ix_crank = await crank(stakePoolKey, programId);
  tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_crank,
  ]);

  /**
   * Verifications
   */

  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  centralStateObj = await CentralState.retrieve(connection, centralKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  expect(
    stakePoolObj.balances.filter((b) => !b.poolReward.isZero()).length
  ).toBe(1);
  expect(
    stakePoolObj.balances
      .filter((b) => !b.poolReward.isZero())[0]
      .poolReward.toString()
  ).toBe(
    new BN(dailyInflation)
      .mul(new BN(stakeAmount))
      .mul(new BN(2 ** 32))
      .mul(new BN(100).sub(stakePoolObj.stakersPart))
      .div(centralStateObj.totalStaked)
      .div(new BN(100))
      .toString()
  );

  /**
   * Claim bond rewards
   */

  let ix_claim_bond_rewards = await claimBondRewards(
    connection,
    bondKey,
    stakerAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_claim_bond_rewards]
  );

  // Verifications
  bondObj = await BondAccount.retrieve(connection, bondKey);

  expect(bondObj.tag).toBe(Tag.BondAccount);
  expect(bondObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(bondObj.totalAmountSold.toNumber()).toBe(bondAmount);
  expect(bondObj.totalStaked.toNumber()).toBe(0);
  expect(bondObj.totalQuoteAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.quoteMint.toBase58()).toBe(
    quoteToken.token.publicKey.toBase58()
  );
  expect(bondObj.sellerTokenAccount.toBase58()).toBe(quoteSellerAta.toBase58());
  expect(bondObj.unlockStartDate.toNumber()).toBe(0);
  expect(bondObj.unlockPeriod.toNumber()).toBe(1);
  expect(bondObj.unlockAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.lastUnlockTime.toNumber()).toBeLessThan(now);
  expect(bondObj.totalUnlockedAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.poolMinimumAtCreation.toNumber()).toBe(minimumStakeAmount);
  expect(bondObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(bondObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(bondObj.sellers.length).toBe(1);

  // Claim pool rewards

  preBalance = (await connection.getTokenAccountBalance(stakePoolAta)).value
    .amount;
  expect(preBalance).toBe(new BN(0).toString());

  let ix_claim_pool_rewards = await claimPoolRewards(
    connection,
    stakePoolKey,
    stakePoolAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [stakePoolOwner],
    feePayer,
    [ix_claim_pool_rewards]
  );

  /**
   * Verifications
   */

  // Check post balances

  postBalance = (await connection.getTokenAccountBalance(stakePoolAta)).value
    .amount;
  stakedAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);

  let pool_rewards = new BN(dailyInflation)
    .mul(new BN(stakePoolObj.totalStaked))
    .div(centralStateObj.totalStaked)
    .mul(new BN(20))
    .div(new BN(100));

  expect(postBalance).toBe(
    new BN(preBalance as string, 10).add(pool_rewards).toString()
  );
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Claim rewards
  preBalance = (await connection.getTokenAccountBalance(stakerAta)).value
    .amount;

  let ix_claim_rewards = await claimRewards(
    connection,
    stakeKey,
    stakerAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_claim_rewards]
  );

  /**
   * Verifications
   */

  postBalance = (await connection.getTokenAccountBalance(stakerAta)).value
    .amount;

  let staker_rewards = new BN(stakePoolObj.totalStaked)
    .shln(32)
    .mul(new BN(dailyInflation))
    .mul(new BN(80))
    .div(new BN(100))
    .div(new BN(centralStateObj.totalStaked))
    .div(new BN(stakePoolObj.totalStaked));

  let reward = new BN(stakedAccountObj.stakeAmount)
    .mul(staker_rewards)
    .shrn(32);

  expect(postBalance).toBe(
    new BN(preBalance as string, 10).add(reward).toString()
  );

  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Check new current supply
  let supply = (await connection.getTokenSupply(accessToken.token.publicKey))
    .value.amount;

  expect(supply).toBe(
    // A full daily inflation as pool owner and staker have claimed + bond amount as bond was claimed
    // Not exactly dailyInflation because of rounding (slightly below)
    reward.add(pool_rewards).add(new BN(bondAmount)).toString()
  );

  // Change inflation
  const ix_change_inflation = await changeInflation(
    connection,
    new BN(stakeAmount).mul(new BN(500_000)),
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [ix_change_inflation]
  );

  /**
   * Verifications
   */

  centralStateObj = await CentralState.retrieve(connection, centralKey);
  expect(centralStateObj.tag).toBe(Tag.CentralState);
  expect(centralStateObj.signerNonce).toBe(centralNonce);
  expect(centralStateObj.dailyInflation.toString()).toBe(
    (stakeAmount * 500_000).toString()
  );
  expect(centralStateObj.tokenMint.toBase58()).toBe(
    accessToken.token.publicKey.toBase58()
  );
  expect(centralStateObj.authority.toBase58()).toBe(
    centralStateAuthority.publicKey.toBase58()
  );

  // Change pool minimum
  const ix_change_pool_minimum = await changePoolMinimum(
    connection,
    stakePoolKey,
    20_000 * decimals,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [stakePoolOwner],
    feePayer,
    [ix_change_pool_minimum]
  );

  /**
   * Verifications
   */

  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(20_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());
  expect(stakePoolObj.stakersPart.toNumber()).toBe(80);

  // Change pool multiplier
  const ix_change_pool_multiplier = await changePoolMultiplier(
    connection,
    stakePoolKey,
    50,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [stakePoolOwner],
    feePayer,
    [ix_change_pool_multiplier]
  );

  /**
   * Verifications
   */

  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(20_000 * decimals);
  expect(stakePoolObj.stakersPart.toNumber()).toBe(50);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Crank
  now = Math.floor(new Date().getTime() / 1_000);
  await sleep(delay / 10);
  ix_crank = await crank(stakePoolKey, programId);
  tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_crank,
  ]);

  /**
   * Verifications
   */

  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(20_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  /**
   * Claim bond rewards
   */

  now = Math.floor(new Date().getTime() / 1_000);
  await sleep(delay / 3);
  ix_claim_bond_rewards = await claimBondRewards(
    connection,
    bondKey,
    stakerAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_claim_bond_rewards]
  );

  // Verifications
  bondObj = await BondAccount.retrieve(connection, bondKey);
  expect(bondObj.tag).toBe(Tag.BondAccount);
  expect(bondObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(bondObj.totalAmountSold.toNumber()).toBe(bondAmount);
  expect(bondObj.totalStaked.toNumber()).toBe(0);
  expect(bondObj.totalQuoteAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.quoteMint.toBase58()).toBe(
    quoteToken.token.publicKey.toBase58()
  );
  expect(bondObj.sellerTokenAccount.toBase58()).toBe(quoteSellerAta.toBase58());
  expect(bondObj.unlockStartDate.toNumber()).toBe(0);
  expect(bondObj.unlockPeriod.toNumber()).toBe(1);
  expect(bondObj.unlockAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.lastUnlockTime.toNumber()).toBeLessThan(now);
  expect(bondObj.totalUnlockedAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.poolMinimumAtCreation.toNumber()).toBe(minimumStakeAmount);
  expect(bondObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(bondObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(bondObj.sellers.length).toBe(1);

  // Claim pool rewards
  await sleep(delay / 10);

  ix_claim_pool_rewards = await claimPoolRewards(
    connection,
    stakePoolKey,
    stakerAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [stakePoolOwner],
    feePayer,
    [ix_claim_pool_rewards]
  );

  /**
   * Verifications
   */

  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(2);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(20_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeLessThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Claim rewards
  ix_claim_rewards = await claimRewards(
    connection,
    stakeKey,
    stakerAta,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_claim_rewards]
  );

  /**
   * Verifications
   */
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(20_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeLessThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Unstake
  let ix_unstake = await unstake(connection, stakeKey, stakeAmount, programId);
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_unstake]
  );

  /**
   * Verifications
   */

  now = Math.floor(new Date().getTime() / 1_000);
  stakedAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  expect(stakedAccountObj.tag).toBe(Tag.StakeAccount);
  expect(stakedAccountObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(stakedAccountObj.stakeAmount.toNumber()).toBe(0);
  expect(stakedAccountObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(stakedAccountObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakedAccountObj.poolMinimumAtCreation.toNumber()).toBe(
    minimumStakeAmount
  );
  expect(stakedAccountObj.pendingUnstakeRequests).toBe(1);
  expect(stakedAccountObj.unstakeRequests[0].amount.toNumber()).toBe(
    stakeAmount
  );
  expect(stakedAccountObj.unstakeRequests[0].time.toNumber()).toBeLessThan(
    now + 7 * 24 * 60 * 60
  );
  expect(JSON.stringify(stakedAccountObj.unstakeRequests.slice(1))).toBe(
    JSON.stringify([
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
      new UnstakeRequest({ time: new BN(MAX_i64), amount: new BN(0) }),
    ])
  );

  /**
   * Verifications
   */
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(3);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(20_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(0);

  /**
   * Admin mint
   */
  const adminMintAmount = 2_000 * decimals;
  const receiver = Keypair.generate();
  const receiverAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    accessToken.token.publicKey,
    receiver.publicKey
  );
  const ix_create_receiver_ata = Token.createAssociatedTokenAccountInstruction(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    accessToken.token.publicKey,
    receiverAta,
    receiver.publicKey,
    feePayer.publicKey
  );
  const ix_admin_mint = await adminMint(
    connection,
    adminMintAmount,
    receiverAta,
    programId
  );

  tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [ix_create_receiver_ata, ix_admin_mint]
  );
  const postBalancesReceiver = (
    await connection.getTokenAccountBalance(receiverAta)
  ).value.amount;
  expect(postBalancesReceiver).toBe(new BN(adminMintAmount).toString());

  // Check current new supply

  const currentSupply = (
    await connection.getTokenSupply(accessToken.token.publicKey)
  ).value.amount;
  // Initial bond amount + admin mint + 2 days for inflation
  // Because of rounding it's slightly below
  let pool_rewards_new_inflation = new BN(stakeAmount)
    .mul(new BN(500_000))
    .mul(new BN(stakeAmount))
    .div(centralStateObj.totalStaked)
    .mul(new BN(20))
    .div(new BN(100));

  let staker_rewards_new_inflation = new BN(stakeAmount)
    .shln(32)
    .mul(new BN(stakeAmount).mul(new BN(500_000)))
    .mul(new BN(80))
    .div(new BN(100))
    .div(new BN(centralStateObj.totalStaked))
    .div(new BN(stakeAmount))
    .mul(new BN(stakeAmount))
    .shrn(32);

  const expectedSupply = reward
    .add(pool_rewards)
    .mul(new BN(2)) // Two days with first inflation value
    .add(pool_rewards_new_inflation)
    .add(staker_rewards_new_inflation)
    .add(new BN(bondAmount))
    .add(new BN(adminMintAmount));
  expect(currentSupply).toBe(expectedSupply.toString());

  /**
   * Freeze the stake pool account
   */

  const ix_freeze_pool = await adminFreeze(connection, stakePoolKey, programId);
  tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [ix_freeze_pool]
  );

  // Verifications
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.FrozenStakePool);

  /**
   * Unfreeze stake pool account
   */

  const ix_unfreeze_pool = await adminFreeze(
    connection,
    stakePoolKey,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [ix_unfreeze_pool]
  );
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
});
