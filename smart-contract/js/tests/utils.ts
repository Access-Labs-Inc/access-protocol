import {
  Keypair,
  PublicKey,
  Connection,
  LAMPORTS_PER_SOL,
  TransactionInstruction,
  Transaction,
} from "@solana/web3.js";
import * as path from "path";
import { readFileSync } from "fs";
import { ChildProcess, spawn, execSync } from "child_process";
import tmp from "tmp";
import { getOrCreateAssociatedTokenAccount, createMint, mintTo } from "@solana/spl-token";

const programName = "access_protocol";

// Spawns a local solana test validator. Caller is responsible for killing the
// process.
export async function spawnLocalSolana(): Promise<ChildProcess> {
  const ledger = tmp.dirSync();
  return spawn("solana-test-validator", ["-l", ledger.name]);
}

// Returns a keypair and key file name.
export function initializePayer(): [Keypair, string] {
  const name = "wallet.json";
  const currentWallet = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(readFileSync(name, "utf-8")))
  );
  return [currentWallet, name];
}

// Deploys the agnostic order book program. Fees are paid with the fee payer
// whose key is in the given key file.
export function deployProgram(
  payerKeyFile: string,
  compile: boolean,
  compileFlag?: string,
  testBpf?: boolean
): PublicKey {
  const programDirectory = path.join(path.dirname(__filename), "../../program");
  const stakingSo = path.join(
    programDirectory,
    `target/deploy/${programName}.so`
  );
  const keyfile = path.join(
    path.dirname(stakingSo),
    `${programName}-keypair.json`
  );
  let compileCmd = "cargo build-bpf";
  if (compileFlag) {
    compileCmd += ` --features ${compileFlag}`;
  }
  if (compile) {
    execSync(compileCmd, {
      cwd: programDirectory,
    });
  }
  if (testBpf) {
    execSync(
      "cargo test-bpf --features days-to-sec-10s no-mint-check no-bond-signer",
      {
        cwd: programDirectory,
      }
    );
  }

  const bytes = readFileSync(keyfile, "utf-8");
  const keypair = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(bytes)));
  execSync(
    [
      "solana program deploy",
      stakingSo,
      "--program-id",
      keyfile,
      "-u https://api.devnet.solana.com",
      "-k",
      payerKeyFile,
      "--commitment finalized",
    ].join(" ")
  );
  return keypair.publicKey;
}

// Funds the given account. Sleeps until the connection is ready.
export async function airdropPayer(connection: Connection, key: PublicKey) {
  while (true) {
    try {
      const signature = await connection.requestAirdrop(
        key,
        2 * LAMPORTS_PER_SOL
      );
      console.log(`Airdrop signature ${signature}`);
      await connection.confirmTransaction(signature, "finalized");
      return;
    } catch (e) {
      console.log(`Error airdropping ${e}`);
      await new Promise((resolve) => setTimeout(resolve, 1000));
      continue;
    }
  }
}

export const signAndSendTransactionInstructions = async (
  // sign and send transaction
  connection: Connection,
  signers: Array<Keypair> | undefined,
  feePayer: Keypair,
  txInstructions: Array<TransactionInstruction>
): Promise<string> => {
  const tx = new Transaction();
  tx.feePayer = feePayer.publicKey;
  signers = signers ? [...signers, feePayer] : [];
  tx.add(...txInstructions);
  const sig = await connection.sendTransaction(tx, signers, {
    skipPreflight: false,
  });
  // await connection.confirmTransaction(sig, "finalized");
  return sig;
};

export class TokenMint {
  token: Keypair;
  connection: Connection;
  feePayer: Keypair;
  mintAuthority: PublicKey;

  constructor(token: Keypair, connection: Connection, feePayer: Keypair, mintAuthority: PublicKey) { 
    this.token = token;
    this.connection = connection;
    this.feePayer = feePayer;
    this.mintAuthority = mintAuthority;
  }

  static async init(
    connection: Connection,
    feePayer: Keypair,
    mintAuthority: PublicKey | null = null
  ) {
    let tokenKeypair = new Keypair();
    await createMint(
      connection,
      feePayer,
      mintAuthority ?? tokenKeypair.publicKey,
      null,
      6,
      tokenKeypair,
    );
    return new TokenMint(tokenKeypair, connection, feePayer, mintAuthority ?? tokenKeypair.publicKey);
  }

  async getAssociatedTokenAccount(wallet: PublicKey): Promise<PublicKey> {
    let acc = await getOrCreateAssociatedTokenAccount(
      this.connection,
      this.feePayer,
      this.token.publicKey,
      wallet
    );
    return acc.address;
  }

  async mintInto(tokenAccount: PublicKey, amount: number): Promise<any> {
    return mintTo(
      this.connection, 
      this.feePayer,
      this.token.publicKey,
      tokenAccount,
      this.mintAuthority,
      amount
    );
  }
}
