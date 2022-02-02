import { useAsyncData } from "../utils/fetch-loop";
import { StakeAccount, ACCESS_PROGRAM_ID } from "@access";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import tuple from "immutable-tuple";
import { PublicKey } from "@solana/web3.js";

export const useStakeAccount = (poolKey: string | null | undefined) => {
  const { connection } = useConnection();
  const { publicKey } = useWallet();
  const fn = async () => {
    if (!poolKey || !publicKey) return;
    const [key] = await StakeAccount.getKey(
      ACCESS_PROGRAM_ID,
      publicKey,
      new PublicKey(poolKey)
    );
    const stakeAccount = await StakeAccount.retrieve(connection, key);
    return stakeAccount;
  };
  return useAsyncData(fn, tuple("useStakeAccount", poolKey, !!publicKey));
};
