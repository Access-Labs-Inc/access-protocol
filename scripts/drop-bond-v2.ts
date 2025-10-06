import { addToBondV2, createBondV2 } from "../smart-contract/js";
import {
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  Signer,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import fs from "fs";
import BN from "bn.js";

// get from env variables
const {
  SOLANA_RPC_PROVIDER_URL,
  PROGRAM_ID,
  POOL_PUBKEY,
  USER_PUBKEY,
  PAYER_KEYPAIR,
  AMOUNT,
  UNLOCK_TIMESTAMP,
} = process.env;

if (!SOLANA_RPC_PROVIDER_URL) {
  throw new Error("SOLANA_RPC_PROVIDER_URL is required");
}
if (!PROGRAM_ID) {
  throw new Error("PROGRAM_ID is required");
}
if (!POOL_PUBKEY) {
  throw new Error("POOL_PUBKEY is required");
}
if (!USER_PUBKEY) {
  throw new Error("USER_PUBKEY is required");
}
if (!PAYER_KEYPAIR) {
  throw new Error("PAYER_KEYPAIR is required");
}
if (!AMOUNT) {
  throw new Error("AMOUNT is required");
}
if (!UNLOCK_TIMESTAMP) {
  throw new Error("UNLOCK_TIMESTAMP is required");
}

const connection = new Connection(SOLANA_RPC_PROVIDER_URL);
const programId = new PublicKey(PROGRAM_ID);
const user = new PublicKey(USER_PUBKEY);
const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(PAYER_KEYPAIR).toString()))
);
const pool = new PublicKey(POOL_PUBKEY);
const amount = parseInt(AMOUNT);
if (isNaN(amount)) {
  throw new Error("AMOUNT must be a number");
}
const unlockTimestamp = parseInt(UNLOCK_TIMESTAMP);
if (isNaN(unlockTimestamp)) {
  throw new Error("UNLOCK_TIMESTAMP must be a number");
}

export const sendIxs = async (connection: Connection, ixs: TransactionInstruction[], signers: Signer[]) => {
  const computePriceIx = ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 10_000_000,
  });

  const computeLimitIx = ComputeBudgetProgram.setComputeUnitLimit({
    units: 300_000,
  });

  const messageV0 = new TransactionMessage({
    payerKey: signers[0].publicKey,
    recentBlockhash: (await connection.getLatestBlockhash('max')).blockhash,
    instructions: [computePriceIx, computeLimitIx, ...ixs],
  }).compileToV0Message();

  const transaction = new VersionedTransaction(messageV0);
  transaction.sign(signers);

  const txl = transaction.serialize().length;
  console.log("Tx size:", txl);

  return await connection.sendTransaction(transaction,
    {
      skipPreflight: false,
    }
  );
}

const main = async () => {
  const createBondIx = createBondV2(
    user,
    payer.publicKey,
    pool,
    new BN.BN(unlockTimestamp),
    programId,
  );

  // Add to bond V2
  const addBondIx = await addToBondV2(
    connection,
    user,
    payer.publicKey,
    pool,
    new BN.BN(amount * 1e6),
    new BN.BN(unlockTimestamp),
    programId,
  );

  const sx = await sendIxs(connection, [createBondIx, addBondIx], [payer]);
  console.log("Transaction:", sx);
};

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
