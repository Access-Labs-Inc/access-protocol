import { CentralState, ACCESS_PROGRAM_ID } from "@access";
import { useAsyncData } from "../utils/fetch-loop";
import { useConnection } from "@solana/wallet-adapter-react";

export const useCentralState = () => {
  const { connection } = useConnection();
  const fn = async () => {
    const [key] = await CentralState.getKey(ACCESS_PROGRAM_ID);
    const centralState = await CentralState.retrieve(connection, key);
    return centralState;
  };
  return useAsyncData(fn, "useCentralState");
};
