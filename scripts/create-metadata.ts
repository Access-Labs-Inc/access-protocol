import fs from "fs";
import bs58 from 'bs58';
import readlineSync from "readline-sync";

import { Connection, LAMPORTS_PER_SOL, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import Solana from "@ledgerhq/hw-app-solana";
import TransportNodeHid from "@ledgerhq/hw-transport-node-hid";
import { createCreateMetadataAccountV2Instruction } from '@metaplex-foundation/mpl-token-metadata';
import { findMetadataPda } from '@metaplex-foundation/js';
import { getWalletOrDefault, getRPCProviderUrl } from './utils';

const { NETWORK, LEDGER_WALLET_BIP32 } = process.env;

const main = async (
  mintWallet: string,
  metadataFilePath: string,
) => {
  // The Solana RPC connection
  const rpcProviderUrl = getRPCProviderUrl(NETWORK);
  const connection = new Connection(rpcProviderUrl);

  const metadata = JSON.parse(fs.readFileSync(metadataFilePath).toString()) as Metadata;

  // Create metadata
  const signature = await updateMetadata(
    connection,
    new PublicKey(mintWallet),
    metadata
  );

  console.log(`Metadata created successfully with signature: ${signature}`);
};

// Run:
// NETWORK=devnet yarn create-metadata MINT_ADDRESS metadata.json
main(process.argv[2], process.argv[3])
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });