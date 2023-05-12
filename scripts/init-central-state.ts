import fs from "fs";
import { Connection, Keypair, PublicKey, TransactionMessage, VersionedTransaction } from "@solana/web3.js";

import { createCentralState, } from "../smart-contract/js/dist";

const {
  SOLANA_RPC_PROVIDER_URL, PROGRAM_PUBKEY, AUTHORITY_KEYPAIR, MINT_ADDRESS, YEARLY_INFLATION_IN_ACS
} = process.env;

if (SOLANA_RPC_PROVIDER_URL == null)
  throw new Error("SOLANA_RPC_PROVIDER_URL must be set.");
if (PROGRAM_PUBKEY == null)
  throw new Error("PROGRAM_PUBKEY must be set.");
if (AUTHORITY_KEYPAIR == null)
  throw new Error("AUTHORITY_KEYPAIR must be set.");
if (MINT_ADDRESS == null)
  throw new Error("MINT_ADDRESS must be set.");
if (YEARLY_INFLATION_IN_ACS == null)
  throw new Error("YEARLY_INFLATION_IN_ACS must be set.");

// The Solana RPC connection
const connection = new Connection(SOLANA_RPC_PROVIDER_URL);

// The wallet used to initialize the central state
// This wallet will be the central state authority
const authorityKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(AUTHORITY_KEYPAIR).toString()))
);

// 🚨 The initial inflation in tokens/day (raw amount i.e need to contain decimals)
const dailyInflation = Math.floor((parseInt(YEARLY_INFLATION_IN_ACS) * (10 ** 6)) / 365);
console.log("Daily inflation at: ", Number(dailyInflation));
console.log("Program ID: ", PROGRAM_PUBKEY);
console.log("Mint address: ", MINT_ADDRESS);

const initCentralState = async () => {
  const ix = await createCentralState(
    Number(dailyInflation),
    authorityKeypair.publicKey, // Central state authority
    authorityKeypair.publicKey, // Fee payer
    new PublicKey(MINT_ADDRESS), // Key to token program
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

  console.log(`Created central state ${tx}`);

  const [centralKey] = PublicKey.findProgramAddressSync(
    [new PublicKey(PROGRAM_PUBKEY).toBuffer()],
    new PublicKey(PROGRAM_PUBKEY)
  );
  // write central state key to file
  fs.writeFileSync("central_state_pubkey.txt", centralKey.toString());
};

initCentralState()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
