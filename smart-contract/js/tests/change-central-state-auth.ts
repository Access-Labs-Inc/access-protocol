import {
  changeCentralStateAuthority,
} from "../src/bindings";
import {
  Connection,
  Keypair,
  PublicKey,
} from "@solana/web3.js";
import { CentralState, Tag } from "../src/state";
import { TokenMint } from "./utils";
import { signAndSendTransactionInstructions } from "./utils";
import { expect } from "@jest/globals";

export const changeCentralStateAuth = async (
  connection: Connection,
  programId: PublicKey,
  feePayer: Keypair,
  centralStateAuthority: Keypair,
  accessToken: TokenMint
) => {
  /**
   * Test variables
   */
  const [centralKey, centralNonce] = await CentralState.getKey(programId);
  const nextAuth = Keypair.generate();

  let cs = await CentralState.retrieve(connection, centralKey);

  expect(cs.authority.toBase58()).toBe(
    centralStateAuthority.publicKey.toBase58()
  );
  expect(cs.signerNonce).toBe(centralNonce);
  expect(cs.tag).toBe(Tag.CentralState);
  expect(cs.tokenMint.toBase58()).toBe(accessToken.token.publicKey.toBase58());

  /**
   * Change central state
   */
  const ix = await changeCentralStateAuthority(
    connection,
    nextAuth.publicKey,
    programId
  );
  const tx = await signAndSendTransactionInstructions(
    connection,
    [centralStateAuthority],
    feePayer,
    [ix]
  );

  console.log("Central state auth changed: ", tx)

  cs = await CentralState.retrieve(connection, centralKey);

  expect(cs.authority.toBase58()).toBe(nextAuth.publicKey.toBase58());
  expect(cs.signerNonce).toBe(centralNonce);
  expect(cs.tag).toBe(Tag.CentralState);
  expect(cs.tokenMint.toBase58()).toBe(accessToken.token.publicKey.toBase58());
};
