import {
  Connection,
  Keypair,
  TransactionInstruction,
  Transaction,
} from "@solana/web3.js";

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
