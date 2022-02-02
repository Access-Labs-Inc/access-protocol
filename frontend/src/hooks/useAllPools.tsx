import { useAsyncData } from "../utils/fetch-loop";
import { getAllStakePools, getAllInactiveStakePools } from "@access";
import { useConnection } from "@solana/wallet-adapter-react";

export const useAllActivePools = () => {
  const { connection } = useConnection();
  const fn = async () => {
    return await getAllStakePools(connection);
  };
  return useAsyncData(fn, "useAllActivePools");
};

export const useAllInactivePools = () => {
  const { connection } = useConnection();
  const fn = async () => {
    return await getAllInactiveStakePools(connection);
  };
  return useAsyncData(fn, "useAllInactivePools");
};
