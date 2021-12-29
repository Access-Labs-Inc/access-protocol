import { useAsyncData } from "../utils/fetch-loop";
import axios from "axios";
import { useWallet } from "@solana/wallet-adapter-react";
import tuple from "immutable-tuple";

export const BACKEND_URL = process.env.REACT_APP_BACKEND;

export interface NonceResult {
  nonce: string;
}

export interface NonceResponse {
  success: boolean;
  result: NonceResult;
}

export const useNonce = () => {
  const { publicKey, connected } = useWallet();

  const fn = async () => {
    if (!connected || !publicKey) return;
    const response = (
      await axios.post(BACKEND_URL + "/auth/nonce", {
        address: publicKey.toBase58(),
      })
    ).data as NonceResponse;
    return response.result;
  };

  return useAsyncData(fn, tuple("useNonce", connected));
};

export interface LoginResult {
  token: string;
}
export interface LoginResponse {
  success: boolean;
  result: LoginResult;
}
