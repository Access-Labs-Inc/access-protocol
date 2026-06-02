import {
  activateStakePool,
  addToBondV2,
  CentralStateV2,
  crank,
  createBondV2,
  StakePool,
  Tag,
} from "../smart-contract/js";
import {
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  ComputeBudgetProgram,
  Connection,
  Keypair,
  PublicKey,
  SendTransactionError,
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

if (unlockTimestamp > 0) {
  const nowTs = Math.floor(Date.now() / 1000);
  if (unlockTimestamp <= nowTs) {
    throw new Error(`UNLOCK_TIMESTAMP is in the past (now=${nowTs}, unlock=${unlockTimestamp}). Set a future unix timestamp or use 0 for no unlock.`);
  }
}

const unlockTsBn = unlockTimestamp > 0 ? new BN(unlockTimestamp) : null;
const amountBn = new BN(amount).mul(new BN(1_000_000));

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

const logSendTransactionError = async (err: unknown) => {
  if (err instanceof SendTransactionError) {
    const logs = await (err as SendTransactionError & {
      getLogs?: (conn: Connection) => Promise<string[]>;
      logs?: string[];
    }).getLogs?.(connection);
    console.error("SendTransactionError logs:");
    console.error(logs ?? (err as { logs?: string[] }).logs ?? []);
    return logs ?? (err as { logs?: string[] }).logs ?? [];
  }
  return [] as string[];
};

const main = async () => {
  const ixs: TransactionInstruction[] = [];

  const [centralState] = CentralStateV2.getKey(programId);
  const centralStateData = await CentralStateV2.retrieve(connection, centralState);
  console.log(centralStateData.tokenMint)
  console.log("payer:", payer.publicKey.toBase58());
  const payerAta = getAssociatedTokenAddressSync(
    centralStateData.tokenMint,
    payer.publicKey,
    true,
  );
  const payerAtaData = await connection.getAccountInfo(payerAta);
  if (!payerAtaData) {
    console.warn("Payer ATA missing. Adding createAssociatedTokenAccount instruction.");
    ixs.push(
      createAssociatedTokenAccountInstruction(
        payer.publicKey,
        payerAta,
        payer.publicKey,
        centralStateData.tokenMint,
      )
    );
  }

  const payerTokenBalance = await connection.getTokenAccountBalance(payerAta).catch(() => null);
  const payerAmountRaw = BigInt(payerTokenBalance?.value.amount ?? "0");
  const requiredAmountRaw = BigInt(amountBn.toString(10));
  if (payerAmountRaw < requiredAmountRaw) {
    throw new Error(
      `Insufficient ACS balance in payer ATA ${payerAta.toBase58()}: have ${payerAmountRaw.toString()}, need ${requiredAmountRaw.toString()}.`
    );
  }

  const poolState = await StakePool.retrieve(connection, pool);
  if (poolState.tag === Tag.InactiveStakePool) {
    console.warn("Stake pool is inactive (tag=2). Adding activateStakePool instruction before bonding.");
    ixs.push(activateStakePool(pool, programId));
  } else if (poolState.tag !== Tag.StakePool) {
    throw new Error(`POOL_PUBKEY is not an active stake pool. Current tag: ${poolState.tag}`);
  }

  // add_to_bond_v2 requires pool current day index to be up to date.
  ixs.push(crank(pool, programId));

  const createBondIx = createBondV2(
    user,
    payer.publicKey,
    pool,
    unlockTsBn,
    programId,
  );

  const addBondIx = await addToBondV2(
    connection,
    user,
    payer.publicKey,
    pool,
    amountBn,
    unlockTsBn,
    programId,
  );

  ixs.push(createBondIx, addBondIx);

  try {
    const sx = await sendIxs(connection, ixs, [payer]);
    console.log("Transaction:", sx);
  } catch (err) {
    await logSendTransactionError(err);
    throw err;
  }
};

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
