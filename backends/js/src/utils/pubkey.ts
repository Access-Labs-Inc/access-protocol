import { PublicKey } from "@solana/web3.js";

/**
 * Verifies if a base58 encoded string is valid public key
 * @param x Base58 encoded public key to validate
 * @returns
 */
export const isValidPubkey = (x: string) => {
  try {
    new PublicKey(x);
    return true;
  } catch (e) {
    return false;
  }
};
