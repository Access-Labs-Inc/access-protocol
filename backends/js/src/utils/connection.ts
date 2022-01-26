import { Connection } from "@solana/web3.js";

const RPC_URL = process.env.RPC_URL!;

export const connection = new Connection(RPC_URL);
