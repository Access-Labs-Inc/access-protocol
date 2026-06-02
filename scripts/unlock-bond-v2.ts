import { BondV2Account, Tag, unlockBondV2 } from "../smart-contract/js";
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

const {
  SOLANA_RPC_PROVIDER_URL,
  PROGRAM_ID,
  BOND_ACCOUNT_PUBKEY,
  USER_PUBKEY,
  POOL_PUBKEY,
  UNLOCK_TIMESTAMP,
  PAYER_KEYPAIR,
} = process.env;

if (!SOLANA_RPC_PROVIDER_URL) {
  throw new Error("SOLANA_RPC_PROVIDER_URL is required");
}
if (!PROGRAM_ID) {
  throw new Error("PROGRAM_ID is required");
}
if (!PAYER_KEYPAIR) {
  throw new Error("PAYER_KEYPAIR is required");
}

if (!BOND_ACCOUNT_PUBKEY && (!USER_PUBKEY || !POOL_PUBKEY || UNLOCK_TIMESTAMP === undefined)) {
  throw new Error(
    "Either BOND_ACCOUNT_PUBKEY is required, or set USER_PUBKEY + POOL_PUBKEY + UNLOCK_TIMESTAMP to derive bond account"
  );
}

const connection = new Connection(SOLANA_RPC_PROVIDER_URL);
const programId = new PublicKey(PROGRAM_ID);
const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(PAYER_KEYPAIR).toString()))
);

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

  const messageV0 = new TransactionMessage({
    payerKey: signers[0].publicKey,
    recentBlockhash: (await rpcConnection.getLatestBlockhash("max")).blockhash,
    instructions: [computePriceIx, computeLimitIx, ...ixs],
  }).compileToV0Message();

  const transaction = new VersionedTransaction(messageV0);
  transaction.sign(signers);

  console.log("Tx size:", transaction.serialize().length);

  return rpcConnection.sendTransaction(transaction, {
    skipPreflight: false,
  });
};

const logSendTransactionError = async (err: unknown) => {
  if (err instanceof SendTransactionError) {
    const logs = await (err as SendTransactionError & {
      getLogs?: (conn: Connection) => Promise<string[]>;
      logs?: string[];
    }).getLogs?.(connection);
    console.error("SendTransactionError logs:");
    console.error(logs ?? (err as { logs?: string[] }).logs ?? []);
    return;
  }
  console.error(err);
};

const main = async () => {
  let bondAccount: PublicKey;
  if (BOND_ACCOUNT_PUBKEY) {
    bondAccount = new PublicKey(BOND_ACCOUNT_PUBKEY);
  } else {
    const user = new PublicKey(USER_PUBKEY!);
    const pool = new PublicKey(POOL_PUBKEY!);
    const unlockTs = Number.parseInt(UNLOCK_TIMESTAMP!, 10);
    if (Number.isNaN(unlockTs)) {
      throw new Error("UNLOCK_TIMESTAMP must be a number when deriving bond account");
    }
    const unlockTsBn = unlockTs > 0 ? new BN(unlockTs) : null;
    [bondAccount] = BondV2Account.getKey(programId, user, pool, unlockTsBn);
    console.log("Derived bond account:", bondAccount.toBase58());
  }

  const bondAccountInfo = await connection.getAccountInfo(bondAccount);
  if (!bondAccountInfo) {
    throw new Error(`Bond account not found: ${bondAccount.toBase58()}`);
  }
  if (!bondAccountInfo.owner.equals(programId)) {
    throw new Error(
      `Invalid bond account owner for ${bondAccount.toBase58()}: expected ${programId.toBase58()}, got ${bondAccountInfo.owner.toBase58()}`
    );
  }
  if (bondAccountInfo.data.length < 1 || bondAccountInfo.data[0] !== Tag.BondV2Account) {
    throw new Error(
      `Account ${bondAccount.toBase58()} is not a BondV2 account (tag=${bondAccountInfo.data[0] ?? "n/a"}).` +
      " You may have passed a wallet address instead of a bond account PDA."
    );
  }

  const bond = await BondV2Account.retrieve(connection, bondAccount);
  if (!bond.owner.equals(payer.publicKey)) {
    throw new Error(
      `PAYER_KEYPAIR does not match bond owner. payer=${payer.publicKey.toBase58()} owner=${bond.owner.toBase58()}`
    );
  }

  const unlockIx = await unlockBondV2(connection, bondAccount, programId);

  try {
    const signature = await sendIxs(connection, [unlockIx], [payer]);
    console.log("Bond account:", bondAccount.toBase58());
    console.log("Transaction:", signature);
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