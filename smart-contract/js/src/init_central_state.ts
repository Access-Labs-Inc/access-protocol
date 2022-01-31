import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createCentralState } from "./bindings";
import fs from "fs";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { CentralState } from "./state";
import { signAndSendTransactionInstructions } from "./utils";

// Program ID
const programId = new PublicKey("");
const connection = new Connection("");
const wallet = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("").toString()))
);
const dailyInflation = 10_000;

const initCentralState = async () => {
  // Initialize mint
  const [centralKey] = await CentralState.getKey(programId);
  const token = await Token.createMint(
    connection,
    wallet,
    centralKey,
    null,
    6, // Decimals of the token
    TOKEN_PROGRAM_ID
  );
  // Create central state
  const ix = await createCentralState(
    dailyInflation,
    wallet.publicKey, // Central state authority
    wallet.publicKey,
    token.publicKey,
    programId
  );
  const tx = await signAndSendTransactionInstructions(connection, [], wallet, [
    ix,
  ]);

  console.log(`Create central state ${tx}`);
};

initCentralState();
