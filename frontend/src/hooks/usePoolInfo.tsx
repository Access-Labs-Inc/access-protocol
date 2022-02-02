import { useAsyncData } from "../utils/fetch-loop";
import { useConnection } from "@solana/wallet-adapter-react";
import tuple from "immutable-tuple";
import { StakePool } from "@access";
import { PublicKey } from "@solana/web3.js";

export const usePoolInfo = (poolKey: string | undefined | null) => {
  const { connection } = useConnection();
  const fn = async () => {
    if (!poolKey) return;
    const pool = await StakePool.retrieve(connection, new PublicKey(poolKey));
    return pool;
  };

  return useAsyncData(fn, tuple("usePoolInfo", poolKey));
};
