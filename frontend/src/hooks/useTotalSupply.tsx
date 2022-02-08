import { useAsyncData } from "../utils/fetch-loop";
import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import tuple from "immutable-tuple";

export const useTotalSupply = (mint: PublicKey | null | undefined) => {
  const { connection } = useConnection();
  const fn = async () => {
    if (!mint) return;
    const supply = await connection.getTokenSupply(mint);
    return supply.value.uiAmount;
  };
  return useAsyncData(fn, tuple("useTotalSupply", mint));
};
