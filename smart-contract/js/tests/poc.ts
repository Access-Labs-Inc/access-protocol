import {
  createCentralState,
  createStakeAccount,
  createStakePool,
  stake,
  crank,
  claimRewards,
  adminMint,
  activateStakePool,
} from "../src/bindings";
import {
  Connection,
  Keypair,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import { CentralState, StakePool, StakeAccount } from "../src/state";
import { TokenMint } from "./utils";
import { sleep } from "../src/utils";
import { airdropPayer, signAndSendTransactionInstructions } from "./utils";
import {
  Token,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export const poc = async (
  connection: Connection,
  programId: PublicKey,
  feePayer: Keypair
) => {
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
  const Bob = Keypair.generate();
  const Alice = Keypair.generate();
  let minimumStakeAmount = 10_000 * decimals;
  const bondAmount = 5_000_000 * decimals;
  const bondSeller = Keypair.generate();
  let fees = 0; // Fees collected by the central state
  let FEES = 1 / 100; // % of fees collected on each stake

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

  const bobAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    accessToken.token.publicKey,
    Bob.publicKey
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      accessToken.token.publicKey,
      bobAta,
      Bob.publicKey,
      feePayer.publicKey
    ),
  ]);
  const aliceAta = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    accessToken.token.publicKey,
    Alice.publicKey
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      accessToken.token.publicKey,
      aliceAta,
      Alice.publicKey,
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

  /**
   * Create stake pool
   */
  const [stakePoolKey] = await StakePool.getKey(
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

  // Create stake account and mint
  const tokenAmount = 500_000_000 * Math.pow(10, 6);
  const [aliceStakeKey] = await StakeAccount.getKey(
    programId,
    Alice.publicKey,
    stakePoolKey
  );
  const [bobStakeKey] = await StakeAccount.getKey(
    programId,
    Bob.publicKey,
    stakePoolKey
  );

  tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [
      await createStakeAccount(
        stakePoolKey,
        Alice.publicKey,
        feePayer.publicKey,
        programId
      ),
      await createStakeAccount(
        stakePoolKey,
        Bob.publicKey,
        feePayer.publicKey,
        programId
      ),
      await adminMint(connection, tokenAmount, aliceAta, programId),
      await adminMint(connection, tokenAmount, bobAta, programId),
    ]
  );

  console.log(tx);

  /**
   * Stake
   */

  tx = await signAndSendTransactionInstructions(
    connection,
    [Alice, Bob],
    feePayer,
    [
      await stake(
        connection,
        aliceStakeKey,
        aliceAta,
        tokenAmount / 2,
        programId
      ),
      await stake(connection, bobStakeKey, bobAta, tokenAmount / 2, programId),
    ]
  );

  console.log(tx);

  /**
   * - Crank
   * - Bob claims
   * - Wait 10s (i.e 24h)
   * - Repeat 5 times
   */
  let bobAcc = await connection.getParsedAccountInfo(bobAta);
  let aliceAcc = await connection.getParsedAccountInfo(aliceAta);

  // @ts-expect-error
  const bobBefore = bobAcc.value.data.parsed.info.tokenAmount.uiAmount;
  // @ts-expect-error
  const aliceBefore = aliceAcc.value.data.parsed.info.tokenAmount.uiAmount;

  for (let i = 0; i < 5; i++) {
    await sleep(10_500);
    const ixs: TransactionInstruction[] = [];
    const signers = [Bob];

    ixs.push(
      await crank(stakePoolKey, programId),
      await claimRewards(connection, bobStakeKey, bobAta, programId)
    );

    if (i === 4) {
      console.log("Alice claiming");
      signers.push(Alice);
      ixs.push(
        await claimRewards(connection, aliceStakeKey, aliceAta, programId)
      );
    }

    tx = await signAndSendTransactionInstructions(
      connection,
      signers,
      feePayer,
      ixs
    );

  }
  console.log(`All claimed ${tx}`);

  /**
   * Compare balances
   */
  bobAcc = await connection.getParsedAccountInfo(bobAta);
  aliceAcc = await connection.getParsedAccountInfo(aliceAta);
  // @ts-expect-error
  const bobAfter = bobAcc.value.data.parsed.info.tokenAmount.uiAmount;
  // @ts-expect-error
  const aliceAfter = aliceAcc.value.data.parsed.info.tokenAmount.uiAmount;

  console.log(
    `Bob: before ${bobBefore} - after ${bobAfter} - diff ${bobAfter - bobBefore
    }`
  );

  console.log(
    `Alice: before ${aliceBefore} - after ${aliceAfter} - diff ${aliceAfter - aliceBefore
    }`
  );

  await sleep(10 * 60 * 10_0000);
};
