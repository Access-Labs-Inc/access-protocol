import { createStakePool, StakePool } from "../smart-contract/js";
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

const {
  SOLANA_RPC_PROVIDER_URL,
  PROGRAM_ID,
  OWNER_KEYPAIR,
  PAYER_KEYPAIR,
  MINIMUM_STAKE_AMOUNT,
} = process.env;

if (!SOLANA_RPC_PROVIDER_URL) {
  throw new Error("SOLANA_RPC_PROVIDER_URL is required");
}
if (!PROGRAM_ID) {
  throw new Error("PROGRAM_ID is required");
}
if (!OWNER_KEYPAIR) {
  throw new Error("OWNER_KEYPAIR is required");
}
if (!MINIMUM_STAKE_AMOUNT) {
  throw new Error("MINIMUM_STAKE_AMOUNT is required");
}

const minimumStakeAmount = Number.parseInt(MINIMUM_STAKE_AMOUNT, 10);
if (!Number.isFinite(minimumStakeAmount) || minimumStakeAmount < 0) {
  throw new Error("MINIMUM_STAKE_AMOUNT must be a non-negative integer");
}

const readKeypair = (path: string) => {
  return Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(path).toString()))
  );
};

const connection = new Connection(SOLANA_RPC_PROVIDER_URL);
const programId = new PublicKey(PROGRAM_ID);
const owner = readKeypair(OWNER_KEYPAIR);
const payer = PAYER_KEYPAIR ? readKeypair(PAYER_KEYPAIR) : owner;

const dedupeSigners = (signers: Signer[]) => {
  const seen = new Set<string>();
  return signers.filter((signer) => {
    const key = signer.publicKey.toBase58();
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
};

const sendIxs = async (
  rpcConnection: Connection,
  ixs: TransactionInstruction[],
  signers: Signer[]
) => {
  const computePriceIx = ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 10_000_000,
  });

  const computeLimitIx = ComputeBudgetProgram.setComputeUnitLimit({
    units: 300_000,
  });

  const blockhash = await rpcConnection.getLatestBlockhash("max");

  const messageV0 = new TransactionMessage({
    payerKey: signers[0].publicKey,
    recentBlockhash: blockhash.blockhash,
    instructions: [computePriceIx, computeLimitIx, ...ixs],
  }).compileToV0Message();

  const tx = new VersionedTransaction(messageV0);
  tx.sign(signers);

  console.log("Tx size:", tx.serialize().length);

  return rpcConnection.sendTransaction(tx, {
    skipPreflight: false,
  });
};

const main = async () => {
  const [stakePool] = StakePool.getKey(programId, owner.publicKey);
  const ixs = await createStakePool(
    connection,
    owner.publicKey,
    minimumStakeAmount,
    payer.publicKey,
    programId
  );

  const signature = await sendIxs(connection, ixs, dedupeSigners([payer, owner]));

  console.log("Stake pool:", stakePool.toBase58());
  console.log("Owner:", owner.publicKey.toBase58());
  console.log("Payer:", payer.publicKey.toBase58());
  console.log("Minimum stake amount:", minimumStakeAmount);
  console.log("Transaction:", signature);
};

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });