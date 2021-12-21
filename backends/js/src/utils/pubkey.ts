import { PublicKey } from "@solana/web3.js";

export const isValidPubkey = (x: string) => {
  try {
    new PublicKey(x);
    return true;
  } catch (e) {
    return false;
  }
};
