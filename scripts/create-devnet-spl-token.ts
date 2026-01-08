import fs from "fs";
import readline from "readline";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  TransactionMessage,
  VersionedTransaction
} from "@solana/web3.js";
import { createMint } from "@solana/spl-token";

import { keypairIdentity, Metaplex } from '@metaplex-foundation/js';
import { createCreateMetadataAccountV3Instruction } from '@metaplex-foundation/mpl-token-metadata';

const REQUIRED_SOL = 0.5;

const waitForEnter = async (message: string): Promise<void> => {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  return new Promise(resolve => {
    rl.question(message, () => { rl.close(); resolve(); });
  });
};

const waitForBalance = async (
  connection: Connection,
  publicKey: PublicKey,
  required: number,
  timeoutMs: number = 30000
): Promise<number> => {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const balance = await connection.getBalance(publicKey) / LAMPORTS_PER_SOL;
    if (balance >= required) {
      return balance;
    }
    const elapsed = Math.floor((Date.now() - start) / 1000);
    process.stdout.write(`\r  Waiting for balance... ${elapsed}s`);
    await new Promise(r => setTimeout(r, 2000));
  }
  process.stdout.write('\n');
  return await connection.getBalance(publicKey) / LAMPORTS_PER_SOL;
};

const ensureFunded = async (connection: Connection, publicKey: PublicKey, label: string): Promise<void> => {
  let balance = await connection.getBalance(publicKey) / LAMPORTS_PER_SOL;

  if (balance >= REQUIRED_SOL) {
    console.log(`✓ ${label} balance: ${balance.toFixed(4)} SOL`);
    return;
  }

  console.log(`Attempting airdrop to ${label}...`);
  try {
    const sig = await connection.requestAirdrop(publicKey, LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig);
    console.log(`✓ Airdrop successful`);
    balance = await waitForBalance(connection, publicKey, REQUIRED_SOL, 10000);
  } catch (e) {
    console.log(`✗ Airdrop failed (rate limited or unavailable)`);
  }

  if (balance < REQUIRED_SOL) {
    console.log(`\n════════════════════════════════════════════════════════════`);
    console.log(`  Insufficient balance: ${balance.toFixed(4)} SOL (need ${REQUIRED_SOL} SOL)`);
    console.log(`  Please fund this wallet manually:`);
    console.log(`\n    ${publicKey.toBase58()}`);
    console.log(`\n  Use: https://faucet.solana.com`);
    console.log(`════════════════════════════════════════════════════════════\n`);
    await waitForEnter("Press ENTER after funding...");

    balance = await waitForBalance(connection, publicKey, REQUIRED_SOL, 30000);
    if (balance < REQUIRED_SOL) {
      throw new Error(`Still insufficient balance after 30s: ${balance.toFixed(4)} SOL`);
    }
  }
  console.log(`✓ ${label} balance: ${balance.toFixed(4)} SOL`);
};


const createDevnetSplToken = async (
  connection: Connection,
  decimals: number,
  mintAuthority: Keypair,
) => {
  const mintAuthorityKey = mintAuthority.publicKey;
  console.log(`Mint authority key: ${mintAuthorityKey.toBase58()}`);

  const balance = await connection.getBalance(mintAuthorityKey);
  console.log(`Balance of mint authority wallet: ${(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);

  return await createMint(
    connection,
    mintAuthority,
    mintAuthority.publicKey,
    null,
    decimals
  );
}


const updateMetadata = async (
  connection: Connection,
  mintAddress: PublicKey,
  mintAuthority: Keypair,
  metadata: Metadata,
) => {
  const mintAuthorityKey = await mintAuthority.publicKey;
  const metaplex = Metaplex.make(connection).use(keypairIdentity(mintAuthority))
  const metadataPDA = metaplex.nfts().pdas().metadata({ mint: mintAddress });

  console.log("Metadata: ", metadata);

  // create v0 compatible message
  const messageV0 = new TransactionMessage({
    payerKey: mintAuthorityKey,
    recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
    instructions: [createCreateMetadataAccountV3Instruction({
      metadata: metadataPDA,
      mint: mintAddress,
      mintAuthority: mintAuthorityKey,
      payer: mintAuthorityKey,
      updateAuthority: mintAuthorityKey,
    },
      {
        createMetadataAccountArgsV3:
        {
          data: {
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            sellerFeeBasisPoints: 0,
            creators: null,
            collection: null,
            uses: null
          },
          isMutable: true,
          collectionDetails: null
        },
      })
    ],
  }).compileToV0Message();

  const transaction = new VersionedTransaction(messageV0);
  transaction.sign([mintAuthority]);

  await connection.sendTransaction(transaction, {
    preflightCommitment: "confirmed",
    skipPreflight: false
  });
}

// Decimals of the ACS token
const TOKEN_DECIMALS = 6;

const main = async () => {
  console.log("Minting a new SPL token");
  // The Solana RPC connection
  const rpcProviderUrl = process.env.SOLANA_RPC_PROVIDER_URL || 'https://api.devnet.solana.com';
  console.log("RPC provider: ", rpcProviderUrl);
  const connection = new Connection(rpcProviderUrl);

  // Mint authority keypair - use existing or generate new
  const keypairPath = 'artifacts/spl_authority.json';
  let authorityKeypair: Keypair;

  if (fs.existsSync(keypairPath)) {
    const secretKey = Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, 'utf-8')));
    authorityKeypair = Keypair.fromSecretKey(secretKey);
    console.log(`Using existing Mint Authority Wallet from ${keypairPath}: ${authorityKeypair.publicKey.toBase58()}`);
  } else {
    authorityKeypair = Keypair.generate();
    console.log(`Created new Mint Authority Wallet into ${keypairPath}: ${authorityKeypair.publicKey.toBase58()}`);
    fs.writeFileSync(keypairPath,
      JSON.stringify(Array.from(authorityKeypair.secretKey))
    );
  }

  await ensureFunded(connection, authorityKeypair.publicKey, "Mint authority");

  // Initialize mint
  const tokenPubkey = await createDevnetSplToken(
    connection,
    TOKEN_DECIMALS, // Decimals of the token
    authorityKeypair, // mint authority keypair
  );

  console.log(`Token initiated successfully on address ${tokenPubkey.toBase58()}`);
  // write token address to file mint_address.txt
  fs.writeFileSync(`artifacts/mint_address.txt`, tokenPubkey.toBase58());

  await updateMetadata(connection, tokenPubkey, authorityKeypair, {
    name: "Access Protocol",
    symbol: "ACS",
    image: "https://ap-staging.fra1.digitaloceanspaces.com/1663691449945",
    uri: "https://accessprotocol.s3.eu-central-1.amazonaws.com/testing_token.json",
  });
};


type Metadata = {
  name: string,
  symbol: string,
  image: string,
  uri: string,
}

// Run:
// NETWORK=devnet yarn create-spl-token [EXISTING_WALLET_PATH]
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });