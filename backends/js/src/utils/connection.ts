import { Connection } from "@solana/web3.js";

/**
 * RPC URL e.g https://solana-api.projectserum.com (must be defined in the .env file)
 */
const RPC_URL = process.env.RPC_URL!;

export const connection = new Connection(RPC_URL);
