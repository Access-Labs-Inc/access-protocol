import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createCentralState } from "./bindings";
import fs from "fs";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { CentralState } from "./state";
import { signAndSendTransactionInstructions } from "./utils";

/**
 * After deploying the program on-chain, one needs to initialize the central state
 *
 * This JS script can be used to initialize the central state.
 *
 * You will have to specify the below variables
 *
 * - Program ID: Obtained after deploying the program
 * - Connection: The Solana RPC connection. Try first on devnet
 * - Wallet: The wallet used to pay for gas and used as the central state authority
 * - Daily inflation: In raw token amount per day
 * - Token decimals: Decimals of the ACCS token
 */

// Program ID
const programId = new PublicKey("");

// The Solana RPC connection
const connection = new Connection("");

// The wallet used to initialize the central state
// This wallet will be the central state authority
const wallet = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("").toString()))
);

// 🚨 The initial inflation in tokens/day (raw amount i.e need to contain decimals)
const dailyInflation = 10_000;

// Decimals of the ACCS token
const tokenDecimals = 6;

const initCentralState = async () => {
  // Initialize mint
  const [centralKey] = await CentralState.getKey(programId);
  const token = await Token.createMint(
    connection,
    wallet,
    centralKey,
    null,
    tokenDecimals, // Decimals of the token
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
