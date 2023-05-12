import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Signer,
  TransactionMessage,
  VersionedTransaction
} from "@solana/web3.js";
import { AuthorityType, createSetAuthorityInstruction } from "@solana/spl-token";
import fs from "fs";

const {
  SOLANA_RPC_PROVIDER_URL, SPL_AUTHORITY_KEYPAIR, MINT_ADDRESS, NEW_AUTHORITY_ADDRESS
} = process.env;

if (SOLANA_RPC_PROVIDER_URL == null)
  throw new Error("SOLANA_RPC_PROVIDER_URL must be set.");
if (SPL_AUTHORITY_KEYPAIR == null)
  throw new Error("SPL_AUTHORITY_KEYPAIR must be set.");
if (MINT_ADDRESS == null)
  throw new Error("MINT_ADDRESS must be set.");
if (NEW_AUTHORITY_ADDRESS == null)
  throw new Error("NEW_AUTHORITY_ADDRESS must be set.");

const setNewAuthority = async (
  connection: Connection,
  mintAddress: PublicKey,
  mintAuthority: Signer,
  newAuthorityWallet: PublicKey,
) => {
  const mintAuthorityKey = mintAuthority.publicKey;
  console.log(`Mint authority key: ${mintAuthorityKey.toBase58()}`);

  const balance = await connection.getBalance(mintAuthorityKey);
  console.log(`Balance of mint authority wallet: ${(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);

  const messageV0 = new TransactionMessage({
      payerKey: mintAuthorityKey,
      recentBlockhash: (await connection.getLatestBlockhash()).blockhash,
      instructions: [
        createSetAuthorityInstruction(
          mintAddress,
          mintAuthorityKey,
          AuthorityType.MintTokens,
          newAuthorityWallet,
        )
      ]
    }
  ).compileToV0Message();

  const transaction = new VersionedTransaction(messageV0);
  transaction.sign([mintAuthority]);

  await connection.sendTransaction(transaction, {
    preflightCommitment: "confirmed",
    skipPreflight: false
  });
}

const main = async () => {
  const mintWallet = MINT_ADDRESS
  const newAuthorityWallet = NEW_AUTHORITY_ADDRESS
  console.log("args", mintWallet, newAuthorityWallet);
  // The Solana RPC connection
  const connection = new Connection(SOLANA_RPC_PROVIDER_URL);

  const authorityKeypair = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(SPL_AUTHORITY_KEYPAIR).toString()))
  );

  // Set authority
  const signature = await setNewAuthority(
    connection,
    new PublicKey(mintWallet),
    authorityKeypair,
    new PublicKey(newAuthorityWallet)
  );

  console.log(`New authority set successfully with signature: ${signature}`);
};

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });