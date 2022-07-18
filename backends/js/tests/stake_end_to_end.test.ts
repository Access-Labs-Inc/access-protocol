import { afterAll, beforeAll, expect, jest, test } from "@jest/globals";
import { ChildProcess } from "child_process";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  airdropPayer,
  deployProgram,
  initializePayer,
  spawnLocalSolana,
  signAndSendTransactionInstructions,
  TokenMint
} from "./utils";
import {
  createCentralState,
  createStakeAccount,
  createStakePool,
  createBond,
  activateStakePool,
  closeStakePool,
} from "@access-protocol";
import {
  CentralState,
  Tag,
  StakePool,
  StakeAccount,
  UnstakeRequest,
} from "@access-protocol";
import {
  Token,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

import { checkStake } from "../src/utils/stake"

import BN from "bn.js";

// Global state initialized once in test startup and cleaned up at test
// teardown.
let solana: ChildProcess;
let connection: Connection;
let feePayer: Keypair;
let payerKeyFile: string;
let programId: PublicKey;
const MAX_i64 = "9223372036854775807";

beforeAll(async () => {
  solana = await spawnLocalSolana();
  connection = new Connection("http://localhost:8899", "finalized");
  [feePayer, payerKeyFile] = initializePayer();
  await airdropPayer(connection, feePayer.publicKey);
  programId = deployProgram(
    payerKeyFile,
    true,
    "days-to-sec-10s no-mint-check no-bond-signer",
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

  const checkStakeResult = await checkStake(staker.publicKey.toBase58());
  expect(checkStakeResult).toBe(true);

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
