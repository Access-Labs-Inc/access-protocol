import { afterAll, beforeAll, expect, jest, test } from "@jest/globals";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  airdropPayer,
  deployProgram,
  initializePayer,
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
  closeStakePool,
  editMetadata,
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
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { sleep } from "../src/utils";
import BN from "bn.js";
import { TokenMint } from "./utils";
import { poc } from "./poc";
import { changeCentralStateAuth } from "./change-central-state-auth";
import { findMetadataPda } from "@metaplex-foundation/js";
import { Metadata } from "@metaplex-foundation/mpl-token-metadata";

// Global state initialized once in test startup and cleaned up at test
// teardown.
let connection: Connection;
let feePayer: Keypair;
let payerKeyFile: string;
let programId: PublicKey;
let accessToken: TokenMint;
const delay = 30_000;
const MAX_i64 = "9223372036854775807";
const centralStateAuthority = Keypair.generate();

beforeAll(async () => {
  connection = new Connection("http://127.0.0.1:8899", "finalized");
  [feePayer, payerKeyFile] = initializePayer();
  await airdropPayer(connection, feePayer.publicKey);
  programId = deployProgram(
    payerKeyFile,
    true,
    "days-to-sec-10s no-mint-check no-bond-signer",
    false
  );
  console.log("Program ID: ", programId.toBase58());
  // get the timestamp of the blockchain
  const slot = await connection.getSlot();
  const timestamp = await connection.getBlockTime(slot);
  console.log("Timestamp: ", timestamp);
});

afterAll(() => {
});

jest.setTimeout(1_500_000);

test("End to end test", async () => {
  // Start time measurement
  const start = Date.now();
  /**
   * Test variables
   */
  const [centralKey, centralNonce] = await CentralState.getKey(programId);
  const decimals = Math.pow(10, 6);
  let dailyInflation = 1_000_000;
  accessToken = await TokenMint.init(connection, feePayer, centralKey);
  const quoteToken = await TokenMint.init(connection, feePayer);
  const stakePoolOwner = Keypair.generate();
  const staker = Keypair.generate();
  let minimumStakeAmount = 10_000 * decimals;
  const bondAmount = 5_000_000 * decimals;
  const bondSeller = Keypair.generate();
  let fees = 0; // Fees collected by the central state
  let FEES = 2 / 100; // % of fees collected on each stake

  await airdropPayer(connection, bondSeller.publicKey);

  /**
   * Set up ATA
   */

  const stakePoolAta = await getAssociatedTokenAddress(
    accessToken.token.publicKey,
    stakePoolOwner.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      stakePoolAta,
      stakePoolOwner.publicKey,
      accessToken.token.publicKey,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    ),
  ]);

  const stakerAta = await getAssociatedTokenAddress(
    accessToken.token.publicKey,
    staker.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      stakerAta,
      staker.publicKey,
      accessToken.token.publicKey,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    ),
  ]);

  const feesAta = await getAssociatedTokenAddress(
    accessToken.token.publicKey,
    centralStateAuthority.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      feesAta,
      centralStateAuthority.publicKey,
      accessToken.token.publicKey,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
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
    "ACCESS",
    "ACCS",
    "some_uri",
    programId
  );

  let tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_central_state,
  ]);
  console.log(`Created central state ${tx}`);

  // Verifications

  let centralStateObj = await CentralState.retrieve(connection, centralKey);

  const metadatKey = findMetadataPda(accessToken.token.publicKey);
  let metadata = await Metadata.fromAccountAddress(connection, metadatKey);

  expect(centralStateObj.tag).toBe(Tag.CentralState);
  expect(centralStateObj.signerNonce).toBe(centralNonce);
  expect(centralStateObj.dailyInflation.toNumber()).toBe(1_000_000);
  expect(centralStateObj.tokenMint.toBase58()).toBe(
    accessToken.token.publicKey.toBase58()
  );
  expect(centralStateObj.authority.toBase58()).toBe(
    centralStateAuthority.publicKey.toBase58()
  );

  // We slice because the metaplex lib does not remove trailling 0s in the buffer info
  expect(metadata.data.name.slice(0, 6)).toBe("ACCESS");
  expect(metadata.data.symbol.slice(0, 4)).toBe("ACCS");
  expect(metadata.data.uri.slice(0, 8)).toBe("some_uri");

  /**
   * Edit metadata
   */
  console.log("Edit metadata");
  const ix_edit_metadata = await editMetadata(
    connection,
    "new name",
    "new symbol",
    "new uri",
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [ix_edit_metadata]
  );
  console.log(`Edit metadata ${tx}`);

  // Verification
  metadata = await Metadata.fromAccountAddress(connection, metadatKey);
  expect(metadata.data.name.slice(0, 8)).toBe("new name");
  expect(metadata.data.symbol.slice(0, 10)).toBe("new symbol");
  expect(metadata.data.uri.slice(0, 7)).toBe("new uri");

  /**
   * Create stake pool
   */
  console.log("Create stake pool");
  const [stakePoolKey, stakePoolNonce] = await StakePool.getKey(
    programId,
    stakePoolOwner.publicKey
  );
  const vault = await getAssociatedTokenAddress(
    accessToken.token.publicKey,
    stakePoolKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
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
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  /**
   * Activate stake pool
   */
  console.log("Activate stake pool");
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
  console.log(`Activated stake pool ${tx}`);

  //Verification
  stakePoolObj = await StakePool.retrieve(connection, stakePoolKey);
  expect(stakePoolObj.tag).toBe(Tag.StakePool);

  // Create stake account
  const [stakeKey, stakeNonce] = await StakeAccount.getKey(
    programId,
    staker.publicKey,
    stakePoolKey
  );

  /**
   * Create stake account
   */

  console.log("Create stake account");
  const ix_create_stake_acc = await createStakeAccount(
    stakePoolKey,
    staker.publicKey,
    feePayer.publicKey,
    programId
  );
  tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_create_stake_acc,
  ]);
  console.log(`Created stake account ${tx}`);

  /**
   * Verifications
   */

  now = Math.floor(new Date().getTime() / 1_000);
  const stakeAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  expect(stakeAccountObj.tag).toBe(Tag.StakeAccount);
  expect(stakeAccountObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(stakeAccountObj.stakeAmount.toNumber()).toBe(0);
  expect(stakeAccountObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(stakeAccountObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(stakeAccountObj.poolMinimumAtCreation.toNumber()).toBe(
    minimumStakeAmount
  );

  /**
   * Create a bond
   */

  console.log("Create bond account ATAs");
  const quoteBuyerAta = await getAssociatedTokenAddress(
    quoteToken.token.publicKey,
    staker.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  const ix_quote_buyer_ata = createAssociatedTokenAccountInstruction(
    feePayer.publicKey,
    quoteBuyerAta,
    staker.publicKey,
    quoteToken.token.publicKey,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  const quoteSellerAta = await getAssociatedTokenAddress(
    quoteToken.token.publicKey,
    bondSeller.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  const ix_quote_seller_ata = createAssociatedTokenAccountInstruction(
    feePayer.publicKey,
    quoteSellerAta,
    bondSeller.publicKey,
    quoteToken.token.publicKey,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix_quote_buyer_ata,
    ix_quote_seller_ata,
  ]);
  console.log("Created bond account ATAs", tx);

  await quoteToken.mintInto(quoteBuyerAta, bondAmount);

  const [bondKey] = await BondAccount.getKey(
    programId,
    staker.publicKey,
    bondAmount
  );
  console.log("Create bond account");
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
  console.log("Created bond account ", tx);

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
  expect(bondObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(bondObj.sellers.length).toBe(1);
  expect(bondObj.sellers[0].toBase58()).toBe(bondSeller.publicKey.toBase58());

  /**
   * Claim bond
   */

  console.log("Claim bond");
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
  console.log("Claimed bond ", tx);

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
  expect(bondObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(bondObj.sellers.length).toBe(1);
  expect(bondObj.sellers[0].toBase58()).toBe(bondSeller.publicKey.toBase58());

  /**
   * Unlock bond tokens
   */

  console.log("Unlock bond tokens");
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
  console.log("Unlocked bond tokens", tx);

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
  expect(bondObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(bondObj.sellers.length).toBe(1);
  expect(bondObj.sellers[0].toBase58()).toBe(bondSeller.publicKey.toBase58());

  // Stake
  preBalance = (await connection.getTokenAccountBalance(stakerAta)).value
    .amount;

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

  // print time since start
  console.log("Time since start: ", (new Date().getTime() - start) / 1000, "s");

  const slot = await connection.getSlot();
  const timestamp = await connection.getBlockTime(slot);
  console.log("Timestamp 2: ", timestamp);

  /**
   * Verifications
   */

  fees = Math.floor(stakeAmount * FEES);
  await sleep(5_000);
  postBalance = (await connection.getTokenAccountBalance(stakerAta)).value
    .amount;
  expect(postBalance).toBe(
    new BN(
      new BN(preBalance).sub(new BN(stakeAmount)).sub(new BN(fees))
    ).toString()
  );

  let stakedAccountObj = await StakeAccount.retrieve(connection, stakeKey);
  expect(stakedAccountObj.tag).toBe(Tag.StakeAccount);
  expect(stakedAccountObj.owner.toBase58()).toBe(staker.publicKey.toBase58());
  expect(stakedAccountObj.stakeAmount.toNumber()).toBe(stakeAmount);
  expect(stakedAccountObj.stakePool.toBase58()).toBe(stakePoolKey.toBase58());
  expect(stakedAccountObj.lastClaimedOffset.toNumber()).toBe(1);
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
  expect(stakePoolObj.currentDayIdx).toBe(2);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
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
      .mul(new BN(2 ** 32))
      .mul(new BN(100).sub(stakePoolObj.stakersPart))
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
  expect(bondObj.lastClaimedOffset.toNumber()).toBe(2);
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
    .mul(new BN(50))
    .div(new BN(100));

  expect(postBalance).toBe(
    new BN(preBalance as string, 10).add(pool_rewards).toString()
  );
  expect(stakePoolObj.tag).toBe(Tag.StakePool);
  expect(stakePoolObj.nonce).toBe(stakePoolNonce);
  expect(stakePoolObj.currentDayIdx).toBeGreaterThan(1);
  expect(stakePoolObj.minimumStakeAmount.toNumber()).toBe(10_000 * decimals);
  expect(stakePoolObj.totalStaked.toNumber()).toBe(stakeAmount);
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
  // todo delta checks
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
    programId,
    false
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
    .mul(new BN(50))
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
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
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
    new BN(500_000),
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
    (500_000).toString()
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
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());
  expect(stakePoolObj.stakersPart.toNumber()).toBe(50);

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
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
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
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
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
  expect(bondObj.lastClaimedOffset.toNumber()).toBe(0);
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
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Claim rewards
  ix_claim_rewards = await claimRewards(
    connection,
    stakeKey,
    stakerAta,
    programId,
    false
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
  expect(stakePoolObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(stakePoolObj.owner.toBase58()).toBe(
    stakePoolOwner.publicKey.toBase58()
  );
  expect(stakePoolObj.vault.toBase58()).toBe(vault.toBase58());

  // Unstake
  let ix_unstake = await unstake(connection, stakeKey, stakerAta, stakeAmount, programId);
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
  expect(stakedAccountObj.lastClaimedOffset.toNumber()).toBe(0);
  expect(stakedAccountObj.poolMinimumAtCreation.toNumber()).toBe(
    minimumStakeAmount
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
  const receiverAta = await getAssociatedTokenAddress(
    accessToken.token.publicKey,
    receiver.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  const ix_create_receiver_ata = createAssociatedTokenAccountInstruction(
    feePayer.publicKey,
    receiverAta,
    receiver.publicKey,
    accessToken.token.publicKey,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
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
  let pool_rewards_new_inflation = new BN(500_000)
    .mul(new BN(stakeAmount))
    .div(centralStateObj.totalStaked)
    .mul(new BN(50))
    .div(new BN(100));

  let staker_rewards_new_inflation = new BN(stakeAmount)
    .shln(32)
    .mul(new BN(500_000))
    .mul(new BN(50))
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

  /**
   * Close stake pool
   */
  const ix_close = await closeStakePool(connection, stakePoolKey, programId);
  tx = await signAndSendTransactionInstructions(
    connection,
    [stakePoolOwner],
    feePayer,
    [ix_close]
  );
});

test("Claim different times", async () => {
  await poc(connection, programId, feePayer, centralStateAuthority, accessToken);
});

test("Change central state auth", async () => {
  await changeCentralStateAuth(connection, programId, feePayer, centralStateAuthority, accessToken);
});
