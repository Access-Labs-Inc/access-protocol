import { useAsyncData } from "../utils/fetch-loop";
import { getAllInactiveBonds, getAllActiveBonds, BondAccount } from "@access";
import { useConnection } from "@solana/wallet-adapter-react";

export const useAllInactiveBonds = () => {
  const { connection } = useConnection();
  const fn = async () => {
    const result = await getAllInactiveBonds(connection);
    return result.map((e) => {
      return {
        bond: BondAccount.deserialize(e.account.data),
        key: e.pubkey,
      };
    });
  };

  return useAsyncData(fn, "useAllBonds");
};

export const useAllActiveBonds = () => {
  const { connection } = useConnection();
  const fn = async () => {
    const result = await getAllActiveBonds(connection);
    return result.map((e) => {
      return {
        bond: BondAccount.deserialize(e.account.data),
        key: e.pubkey,
      };
    });
  };

  return useAsyncData(fn, "useAllActiveBonds");
};
