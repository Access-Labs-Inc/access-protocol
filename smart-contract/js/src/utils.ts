import { Connection, Keypair, Transaction, TransactionInstruction, } from "@solana/web3.js";
import { TaggedInstruction } from "./raw_instructions";
import BN from "bn.js";

export async function sleep(ms: number) {
  console.log("Sleeping for ", ms, " ms");
  return await new Promise((resolve) => setTimeout(resolve, ms));
}

export const signAndSendTransactionInstructions = async (
  // sign and send transaction
  connection: Connection,
  signers: Array<Keypair>,
  feePayer: Keypair,
  txInstructions: Array<TransactionInstruction>
): Promise<string> => {
  const tx = new Transaction();
  tx.feePayer = feePayer.publicKey;
  signers.push(feePayer);
  tx.add(...txInstructions);
  return await connection.sendTransaction(tx, signers, {
    skipPreflight: false,
  });
};

export const getUnfreezeMask = (ixs: TaggedInstruction[]) => {
  if (ixs.length === 0) {
    return new BN(0).notn(128);
  }
  return ixs.reduce((acc, ix) =>
    acc.or(new BN(1).shln(ix.tag)), new BN(0));
}

export const getFreezeMask = (ixs: TaggedInstruction[]) => {
  return getUnfreezeMask(ixs).notn(128);
}
