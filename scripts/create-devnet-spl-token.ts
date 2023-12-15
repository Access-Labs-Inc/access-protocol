import fs from "fs";
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
  const metadataPDA = metaplex.nfts().pdas().metadata({mint:mintAddress});

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
  const rpcProviderUrl = 'https://api.devnet.solana.com'
  console.log("RPC provider: ", rpcProviderUrl);
  const connection = new Connection(rpcProviderUrl);

  // Mint authority keypair
  const authorityKeypair = Keypair.generate();
  console.log(`Created new Mint Authority Wallet into artifacts/spl_authority.json: ${authorityKeypair.publicKey.toBase58()}`);
  fs.writeFileSync(`artifacts/spl_authority.json`,
    JSON.stringify(authorityKeypair.secretKey
      .toString() //convert secret key to string
      .split(',') //delimit string by commas and convert to an array of strings
      .map(value => Number(value))
    )
  );

  console.log(`Funding mint authority wallet with 1 SOL...`)
  try {
    const airdropSignature = await connection.requestAirdrop(authorityKeypair.publicKey, LAMPORTS_PER_SOL);
    await connection.confirmTransaction(airdropSignature);
  } catch (e) {
    console.error(`Error requesting airdrop, top up the wallet manually: ${e}`);
  }

  console.log(`Waiting for 30 seconds to ensure that the airdrop succeeds...`);
  await new Promise(resolve => setTimeout(resolve, 30000));

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