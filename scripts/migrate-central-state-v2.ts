import fs from "fs";
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
  TransactionMessage,
  VersionedTransaction
} from "@solana/web3.js";

import { migrateCentralStateV2, } from "../smart-contract/js";
import { getAssociatedTokenAddress } from "@solana/spl-token/src/state/mint";
import { createAssociatedTokenAccountInstruction, getAssociatedTokenAddressSync } from "@solana/spl-token";

const {
  SOLANA_RPC_PROVIDER_URL, PROGRAM_PUBKEY, AUTHORITY_KEYPAIR, MINT_ADDRESS
} = process.env;

if (SOLANA_RPC_PROVIDER_URL == null)
  throw new Error("SOLANA_RPC_PROVIDER_URL must be set.");
if (PROGRAM_PUBKEY == null)
  throw new Error("PROGRAM_PUBKEY must be set.");
if (AUTHORITY_KEYPAIR == null)
  throw new Error("AUTHORITY_KEYPAIR must be set.");
if (MINT_ADDRESS == null)
  throw new Error("MINT_ADDRESS must be set.");

// The Solana RPC connection
const connection = new Connection(SOLANA_RPC_PROVIDER_URL);

// The wallet used to initialize the central state
// This wallet will be the central state authority
const authorityKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(AUTHORITY_KEYPAIR).toString()))
);

const migrateCentralState = async () => {
  const [centralKey] = PublicKey.findProgramAddressSync(
    [new PublicKey(PROGRAM_PUBKEY).toBuffer()],
    new PublicKey(PROGRAM_PUBKEY)
  );

  try {
    const ata = getAssociatedTokenAddressSync(
      new PublicKey(MINT_ADDRESS), centralKey, true
    );

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        authorityKeypair.publicKey,
        ata,
        centralKey,
        new PublicKey(MINT_ADDRESS),
      )
    );

    await sendAndConfirmTransaction(connection, transaction, [authorityKeypair]);
  } catch (e) {
    console.log("Associated token account not created, it might already exist", e)
  }

  const ix = migrateCentralStateV2(
    authorityKeypair.publicKey, // Central state authority
    new PublicKey(PROGRAM_PUBKEY), // Program ID
  );

  const messageV0 = new TransactionMessage({
    payerKey: authorityKeypair.publicKey,
    recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
    instructions: [ix],
  }).compileToV0Message();

  const transaction = new VersionedTransaction(messageV0);
  transaction.sign([authorityKeypair]);

  const tx = await connection.sendTransaction(transaction, {
    preflightCommitment: "confirmed",
    skipPreflight: false
  });

  console.log(`Migrated central state to v2 ${tx}`);

  // write central state key to file
  fs.writeFileSync("artifacts/central_state_pubkey.txt", centralKey.toString());
};

migrateCentralState()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
