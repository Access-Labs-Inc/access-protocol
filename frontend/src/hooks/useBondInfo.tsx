import { useAsyncData } from "../utils/fetch-loop";
import { PublicKey } from "@solana/web3.js";
import { useConnection } from "@solana/wallet-adapter-react";
import { BondAccount } from "@access";
import tuple from "immutable-tuple";

export const useBondInfo = (bondKey: PublicKey) => {
  const { connection } = useConnection();
  const fn = async () => {
    const bond = await BondAccount.retrieve(connection, bondKey);
    return bond;
  };
  return useAsyncData(fn, tuple("useBondInfo", bondKey.toBase58()));
};
