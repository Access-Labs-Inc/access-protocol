import { Connection, Keypair, PublicKey, Transaction, SystemProgram, sendAndConfirmTransaction } from "@solana/web3.js";
import { createCentralState } from "./bindings";
import fs from "fs";
import { TOKEN_PROGRAM_ID, MintLayout, getMinimumBalanceForRentExemptMint, createInitializeMintInstruction } from "@solana/spl-token";
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

const createMint = async (connection, payer, mintAuthority, freezeAuthority, decimals, tokenKeypair, programId) => {
  const balanceNeeded = await getMinimumBalanceForRentExemptMint(connection);
  const transaction = new Transaction();
  transaction.add(SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: tokenKeypair.publicKey,
    lamports: balanceNeeded,
    space: MintLayout.span,
    programId
  }));
  transaction.add(
    createInitializeMintInstruction(
      tokenKeypair.publicKey,
      decimals,
      mintAuthority,
      freezeAuthority,
      programId
    )
  ); // Send the two instructions

  await sendAndConfirmTransaction(connection, transaction, [payer, tokenKeypair], {
    skipPreflight: false
  });
}

// Program ID
const programId = new PublicKey("acp1VPqNoMs5KC5aEH3MzxnyPZNyKQF1TCPouCoNRuX");

// The Solana RPC connection
const connection = new Connection("");

/**
 * Token metadata
 */
const name = "ACCESS";
const symbol = "ACCS";
const uri = "...";

// The wallet used to initialize the central state
// This wallet will be the central state authority
const authorityKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("").toString()))
);

// The wallet used to for SPL token
const tokenKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("").toString()))
);

// 🚨 The initial inflation in tokens/day (raw amount i.e need to contain decimals)
const dailyInflation = 10_000;

// Decimals of the ACCS token
const tokenDecimals = 6;

const initCentralState = async () => {
  // Initialize mint
  const [centralKey] = await CentralState.getKey(programId);
  await createMint(
    connection,
    authorityKeypair,
    centralKey,
    null,
    tokenDecimals, // Decimals of the token
    tokenKeypair,
    TOKEN_PROGRAM_ID
  );
  // Create central state
  const ix = await createCentralState(
    dailyInflation,
    authorityKeypair.publicKey, // Central state authority
    authorityKeypair.publicKey,
    tokenKeypair.publicKey,
    name,
    symbol,
    uri,
    programId
  );
  const tx = await signAndSendTransactionInstructions(connection, [], authorityKeypair, [
    ix,
  ]);

  console.log(`Create central state ${tx}`);
};

initCentralState();
