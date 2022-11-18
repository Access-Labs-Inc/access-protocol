import {
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
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import { StakePool, StakeAccount } from "../src/state";
import { TokenMint } from "./utils";
import { signAndSendTransactionInstructions, airdropPayer } from "./utils";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { sleep } from "../src/utils";
import { expect } from "@jest/globals";

export const poc = async (
  connection: Connection,
  programId: PublicKey,
  feePayer: Keypair,
  centralStateAuthority: Keypair,
  accessToken: TokenMint,
) => {
  /**
   * Test variables
   */
  const decimals = Math.pow(10, 6);
  const stakePoolOwner = Keypair.generate();
  const Bob = Keypair.generate();
  const Alice = Keypair.generate();
  let minimumStakeAmount = 10_000 * decimals;

  /**
   * Set up ATA
   */

  await airdropPayer(connection, feePayer.publicKey);

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

  const bobAta = await getAssociatedTokenAddress(
    accessToken.token.publicKey,
    Bob.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      bobAta,
      Bob.publicKey,
      accessToken.token.publicKey,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    ),
  ]);
  const aliceAta = await getAssociatedTokenAddress(
    accessToken.token.publicKey,
    Alice.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
  await signAndSendTransactionInstructions(connection, [], feePayer, [
    createAssociatedTokenAccountInstruction(
      feePayer.publicKey,
      aliceAta,
      Alice.publicKey,
      accessToken.token.publicKey,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    ),
  ]);

  /**
   * Create stake pool
   */
  const [stakePoolKey] = await StakePool.getKey(
    programId,
    stakePoolOwner.publicKey
  );
  const ix_stake_pool = await createStakePool(
    connection,
    stakePoolOwner.publicKey,
    minimumStakeAmount,
    feePayer.publicKey,
    programId
  );

  let tx = await signAndSendTransactionInstructions(
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

  const balance = await connection.getBalance(feePayer.publicKey);
  console.log("Balance", balance / LAMPORTS_PER_SOL);

  /**
   * - Crank
   * - Bob claims
   * - Wait 10s (i.e 24h)
   * - Repeat 5 times
   */
  let bobAcc = await connection.getParsedAccountInfo(bobAta);
  let aliceAcc = await connection.getParsedAccountInfo(aliceAta);

  for (let i = 0; i < 5; i++) {
    await sleep(10_500);
    const ixs: TransactionInstruction[] = [];
    const signers = [Bob];

    ixs.push(
      await crank(stakePoolKey, programId),
      await claimRewards(connection, bobStakeKey, bobAta, programId, true)
    );

    if (i === 4) {
      console.log("Alice claiming");
      signers.push(Alice);
      ixs.push(
        await claimRewards(connection, aliceStakeKey, aliceAta, programId, true)
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

  expect(aliceAfter).toBeCloseTo(bobAfter);
};
