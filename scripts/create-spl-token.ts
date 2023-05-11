import fs from "fs";
import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import {
  createInitializeMintInstruction,
  getMinimumBalanceForRentExemptMint,
  MintLayout,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";

import { findMetadataPda, keypairIdentity, Metaplex } from '@metaplex-foundation/js';
import {
  createCreateMetadataAccountV2Instruction,
  DataV2,
  TokenStandard
} from '@metaplex-foundation/mpl-token-metadata';


const createSPLToken = async (
  connection: Connection,
  decimals: number,
  tokenKeypair: Keypair,
  mintAuthority: Keypair,
  programId: PublicKey
) => {
  const mintAuthorityKey = mintAuthority.publicKey;
  console.log(`Mint authority key: ${mintAuthorityKey.toBase58()}`);

  const balance = await connection.getBalance(mintAuthorityKey);
  console.log(`Balance of mint authority wallet: ${(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);

  const balanceNeeded = await getMinimumBalanceForRentExemptMint(connection);
  const transaction = new Transaction();
  transaction.add(SystemProgram.createAccount({
    fromPubkey: mintAuthorityKey,
    newAccountPubkey: tokenKeypair.publicKey,
    lamports: balanceNeeded,
    space: MintLayout.span,
    programId
  }));
  transaction.add(
    createInitializeMintInstruction(
      tokenKeypair.publicKey,
      decimals,
      mintAuthorityKey,
      null,
      programId
    )
  );
  transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  transaction.feePayer = mintAuthorityKey;
  return await connection.sendTransaction(transaction, [mintAuthority, tokenKeypair])
}


const updateMetadata = async (
  connection: Connection,
  mintAddress: PublicKey,
  mintAuthority: Keypair,
  metadata: Metadata,
) => {
  const mintAuthorityKey = await mintAuthority.publicKey;
  console.log(`Mint authority key: ${mintAuthorityKey.toBase58()}`);

  const balance = await connection.getBalance(mintAuthorityKey);
  console.log(`Balance of mint authority wallet: ${(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);

  const metaplex = Metaplex.make(connection).use(keypairIdentity(mintAuthority))

  console.log("Adding metadata: ", metadata);

  const metadataPDA = await findMetadataPda(mintAddress);

  console.log("Metadata: ", metadata);

  const transaction = new Transaction();
  transaction.add(
    createCreateMetadataAccountV2Instruction({
        metadata: metadataPDA,
        mint: mintAddress,
        mintAuthority: mintAuthorityKey,
        payer: mintAuthorityKey,
        updateAuthority: mintAuthorityKey,
      },
      { createMetadataAccountArgsV2:
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
            isMutable: true
          }
      })
  )

  transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
  transaction.feePayer = mintAuthorityKey;

  return await connection.sendTransaction(transaction, [mintAuthority], {
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

  // SPL token keypair - todo solvve the repetition and saving the keypair
  const tokenKeypair = Keypair.generate();
  console.log(`Created new SPL Token Authority Wallet into ${tokenKeypair.publicKey.toBase58()}.json`);
  fs.writeFileSync(`${tokenKeypair.publicKey.toBase58()}.json`,
    JSON.stringify(tokenKeypair.secretKey
      .toString() //convert secret key to string
      .split(',') //delimit string by commas and convert to an array of strings
      .map(value => Number(value))
    )
  );

  // Mint authority keypair
  const authorityKeypair = Keypair.generate();
  console.log(`Created new Mint Authority Wallet into ${authorityKeypair.publicKey.toBase58()}.json`);
  fs.writeFileSync(`${authorityKeypair.publicKey.toBase58()}.json`,
    JSON.stringify(authorityKeypair.secretKey
      .toString() //convert secret key to string
      .split(',') //delimit string by commas and convert to an array of strings
      .map(value => Number(value))
    )
  );

  // Mint authority keypair
  const programKeypair = Keypair.generate();
  console.log(`Created new Program keypair into ${programKeypair.publicKey.toBase58()}.json`);
  fs.writeFileSync(`${programKeypair.publicKey.toBase58()}.json`,
    JSON.stringify(programKeypair.secretKey
      .toString() //convert secret key to string
      .split(',') //delimit string by commas and convert to an array of strings
      .map(value => Number(value))
    )
  );

  console.log(`Funding mint authority wallet with 1 SOL...`)
  const airdropSignature = await connection.requestAirdrop(authorityKeypair.publicKey, LAMPORTS_PER_SOL);
  await connection.confirmTransaction(airdropSignature);

  console.log(`Waiting one minute to ensure that the airdrop succeeds...`);
  await new Promise(resolve => setTimeout(resolve, 60000));

  // Initialize mint
  const signature = await createSPLToken(
    connection,
    TOKEN_DECIMALS, // Decimals of the token
    tokenKeypair, // token keypair
    authorityKeypair, // mint authority keypair
    TOKEN_PROGRAM_ID
  );

  console.log(`Token initiated successfully on address ${tokenKeypair.publicKey.toBase58()}, tx signature: ${signature}`);

  // todo initialize metadata
  await updateMetadata(connection, tokenKeypair.publicKey, authorityKeypair, {
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