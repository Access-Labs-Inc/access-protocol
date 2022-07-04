import {
  createCentralState,
  changeCentralStateAuthority,
} from "../src/bindings";
import {
  Connection,
  Keypair,
  PublicKey,
  SendTransactionError,
} from "@solana/web3.js";
import { CentralState, Tag } from "../src/state";
import { TokenMint } from "./utils";
import { signAndSendTransactionInstructions } from "./utils";
import { expect } from "@jest/globals";

export const changeCentralStateAuth = async (
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
  const nextAuth = Keypair.generate();
  const accessToken = await TokenMint.init(connection, feePayer, centralKey);

  /**
   * Create central state
   */

  let ix = await createCentralState(
    dailyInflation,
    centralStateAuthority.publicKey,
    feePayer.publicKey,
    accessToken.token.publicKey,
    "ACCESS",
    "ACCS",
    "..",
    programId
  );

  let tx = await signAndSendTransactionInstructions(connection, [], feePayer, [
    ix,
  ]);
  console.log(tx);

  let cs = await CentralState.retrieve(connection, centralKey);

  expect(cs.authority.toBase58()).toBe(
    centralStateAuthority.publicKey.toBase58()
  );
  expect(cs.dailyInflation.toNumber()).toBe(dailyInflation);
  expect(cs.signerNonce).toBe(centralNonce);
  expect(cs.tag).toBe(Tag.CentralState);
  expect(cs.tokenMint.toBase58()).toBe(accessToken.token.publicKey.toBase58());
  expect(cs.totalStaked.toNumber()).toBe(0);

  /**
   * Change central state
   */
  ix = await changeCentralStateAuthority(
    connection,
    nextAuth.publicKey,
    programId
  );
  tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [ix]
  );

  cs = await CentralState.retrieve(connection, centralKey);

  expect(cs.authority.toBase58()).toBe(nextAuth.publicKey.toBase58());
  expect(cs.dailyInflation.toNumber()).toBe(dailyInflation);
  expect(cs.signerNonce).toBe(centralNonce);
  expect(cs.tag).toBe(Tag.CentralState);
  expect(cs.tokenMint.toBase58()).toBe(accessToken.token.publicKey.toBase58());
  expect(cs.totalStaked.toNumber()).toBe(0);
};
