import { afterAll, beforeAll, expect, jest, test } from "@jest/globals";
import { ChildProcess } from "child_process";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  airdropPayer,
  deployProgram,
  initializePayer,
  spawnLocalSolana,
  signAndSendTransactionInstructions,
  getTokenAccountBalance,
  getTokenSupply,
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
} from "../src/bindings";
import {
  CentralState,
  Tag,
  StakePool,
  StakeAccount,
  BondAccount,
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

jest.setTimeout(800_000);

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
  const bondAmount = 50_000 * decimals;
  const bondSeller = Keypair.generate();

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
  await sleep(delay);

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
    stakePoolAta,
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
  await sleep(delay);
  let now = new Date().getTime() / 1_000;
  let stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
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

  await sleep(delay);
  now = new Date().getTime() / 1_000;
  const stakeAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  expect(stakeAccountObj.tag).toBe(Tag.StakeAccount);
  expect(stakeAccountObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(stakeAccountObj.stakeAmount.toNumber()).toBe(0);
  expect(stakeAccountObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(stakeAccountObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakeAccountObj.poolMinimumAtCreation.toNumber()).toBe(
    minimumStakeAmount
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

  await sleep(2 * delay);

  await quoteToken.mintInto(quoteBuyerAta, bondAmount);

  await sleep(delay);

  const [bondKey, bondNonce] = await BondAccount.getKey(
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
    1,
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
  await sleep(delay);
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
  expect(bondObj.lastUnlockTime.toNumber()).toBe(1);
  expect(bondObj.totalUnlockedAmount.toNumber()).toBe(0);
  expect(bondObj.poolMinimumAtCreation.toNumber()).toBe(minimumStakeAmount);
  expect(bondObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(bondObj.lastClaimedTime.toNumber()).toBe(0);
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
  await sleep(delay);
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
  expect(bondObj.lastUnlockTime.toNumber()).toBe(1);
  expect(bondObj.totalUnlockedAmount.toNumber()).toBe(0);
  expect(bondObj.poolMinimumAtCreation.toNumber()).toBe(minimumStakeAmount);
  expect(bondObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(bondObj.lastClaimedTime.toNumber()).toBe(0);
  expect(bondObj.sellers.length).toBe(1);
  expect(bondObj.sellers[0].toBase58()).toBe(bondSeller.publicKey.toBase58());

  /**
   * Unlock bond tokens
   */

  let preBalance = await getTokenAccountBalance(connection, stakerAta);
  expect(preBalance).toBe(0);

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
  await sleep(delay);
  now = new Date().getTime() / 1_000;
  bondObj = await BondAccount.retrieve(connection, bondKey);
  let postBalance = await getTokenAccountBalance(connection, stakerAta);
  expect(postBalance).toBe(bondAmount / decimals);

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
  expect(bondObj.lastClaimedTime.toNumber()).toBe(0);
  expect(bondObj.sellers.length).toBe(1);
  expect(bondObj.sellers[0].toBase58()).toBe(bondSeller.publicKey.toBase58());

  // Stake
  preBalance = await getTokenAccountBalance(connection, stakerAta);

  let stakeAmount = 10_000 * decimals;
  let ix_stake = await stake(
    connection,
    stakeKey,
    stakerAta,
    stakeAmount,
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

  now = new Date().getTime() / 1_000;
  await sleep(delay);
  postBalance = await getTokenAccountBalance(connection, stakerAta);
  expect(postBalance).toBe(preBalance - stakeAmount / decimals);

  let stakedAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  expect(stakedAccountObj.tag).toBe(Tag.StakeAccount);
  expect(stakedAccountObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(stakedAccountObj.stakeAmount.toNumber()).toBe(stakeAmount);
  expect(stakedAccountObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(stakedAccountObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakedAccountObj.poolMinimumAtCreation.toNumber()).toBe(
    minimumStakeAmount
  );

  // Crank
  let ix_crank = await crank(stakePoolKey, programId);
  tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_crank,
  ]);

  /**
   * Verifications
   */

  await sleep(delay);
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBe(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  expect(stakePoolObj.balances.filter((b) => !b.isZero()).length).toBe(1);
  expect(stakePoolObj.balances.filter((b) => !b.isZero())[0].toString()).toBe(
    new BN(dailyInflation).mul(new BN(stakeAmount)).toString()
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
  await sleep(delay);
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
  expect(bondObj.lastUnlockTime.toNumber()).toBe(2);
  expect(bondObj.totalUnlockedAmount.toNumber()).toBe(bondAmount);
  expect(bondObj.poolMinimumAtCreation.toNumber()).toBe(minimumStakeAmount);
  expect(bondObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(bondObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(bondObj.sellers.length).toBe(1);

  // The user must have received dailyInflation * stakeAmount * 20 / 100?

  // Claim pool rewards

  preBalance = await getTokenAccountBalance(connection, stakePoolAta);
  expect(preBalance).toBe(0);

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

  await sleep(delay);
  let totalSupply = bondAmount;
  postBalance = await getTokenAccountBalance(connection, stakePoolAta);
  expect(postBalance).toBe(
    preBalance +
      Math.floor((((dailyInflation * stakeAmount) / totalSupply) * 20) / 100)
  );
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBe(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Claim rewards
  preBalance = await getTokenAccountBalance(connection, stakerAta);
  expect(preBalance).toBe((bondAmount - stakeAmount) / decimals);

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

  await sleep(delay);

  postBalance = await getTokenAccountBalance(connection, stakerAta);
  expect(postBalance).toBe(
    preBalance +
      Math.floor((((dailyInflation * stakeAmount) / totalSupply) * 80) / 100)
  );

  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBe(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Check new current supply
  let supply = await getTokenSupply(connection, accessToken.token.publicKey);
  expect(supply).toBe(bondAmount + dailyInflation * stakeAmount);

  // Change inflation
  const ix_change_inflation = await changeInflation(
    connection,
    stakeAmount * 500_000,
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

  await sleep(delay);
  centralStateObj = await CentralState.retrieve(connection, centralKey);
  expect(centralStateObj.tag).toBe(Tag.CentralState);
  expect(centralStateObj.signerNonce).toBe(centralNonce);
  expect(centralStateObj.dailyInflation.toNumber()).toBe(stakeAmount * 500_000);
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

  await sleep(delay);
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBe(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(20_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Crank
  now = new Date().getTime() / 1_000;
  await sleep(delay / 10);
  ix_crank = await crank(stakePoolKey, programId);
  tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_crank,
  ]);

  /**
   * Verifications
   */

  await sleep(delay);
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBe(2);
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

  now = new Date().getTime() / 1_000;
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
  await sleep(delay);
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
  expect(bondObj.lastUnlockTime.toNumber()).toBe(2);
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

  await sleep(delay);
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBe(2);
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
  await sleep(delay);
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBe(2);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(20_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedTime.toNumber()).toBeGreaterThan(now);
  expect(stakePoolObj.lastCrankTime.toNumber()).toBeLessThan(now);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Unstake
  let ix_unstake = await unstake(
    connection,
    stakeKey,
    stakerAta,
    stakeAmount,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [staker],
    feePayer,
    [ix_unstake]
  );

  /**
   * Verifications
   */

  await sleep(delay);
  now = new Date().getTime() / 1_000;
  stakedAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  expect(stakedAccountObj.tag).toBe(Tag.StakeAccount);
  expect(stakedAccountObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(stakedAccountObj.stakeAmount.toNumber()).toBe(0);
  expect(stakedAccountObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(stakedAccountObj.lastClaimedTime.toNumber()).toBeLessThan(now);
  expect(stakedAccountObj.poolMinimumAtCreation.toNumber()).toBe(
    minimumStakeAmount
  );

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
  await sleep(delay);
  const postBalancesReceiver = await getTokenAccountBalance(
    connection,
    receiverAta
  );
  expect(postBalancesReceiver).toBe(adminMintAmount);

  // Check current new supply
});
