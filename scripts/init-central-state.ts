import fs from "fs";
import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";

import { createCentralState, } from "../smart-contract/js/dist";

const {
  SOLANA_RPC_PROVIDER_URL, PROGRAM_PUBKEY, AUTHORITY_KEYPAIR, TOKEN_PUBKEY, YEARLY_INFLATION_IN_ACS
} = process.env;

if (SOLANA_RPC_PROVIDER_URL == null)
  throw new Error("SOLANA_RPC_PROVIDER_URL must be set.");
if (PROGRAM_PUBKEY == null)
  throw new Error("PROGRAM_PUBKEY must be set.");
if (AUTHORITY_KEYPAIR == null)
  throw new Error("AUTHORITY_KEYPAIR must be set.");
if (TOKEN_PUBKEY == null)
  throw new Error("TOKEN_PUBKEY must be set.");
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

const initCentralState = async () => {
  const ix = await createCentralState(
    Number(dailyInflation),
    authorityKeypair.publicKey, // Central state authority
    authorityKeypair.publicKey, // Fee payer
    new PublicKey(TOKEN_PUBKEY), // Key to token program
    new PublicKey(PROGRAM_PUBKEY), // Program ID
  );

  const tx = new Transaction();
  tx.feePayer = authorityKeypair.publicKey;
  tx.add(ix);
  const txId = await connection.sendTransaction(tx, [authorityKeypair], {
    skipPreflight: true,
  });
  console.log(`Create central state ${txId}`);
};

initCentralState()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
