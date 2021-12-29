import crypto from "crypto";
import { PublicKey } from "@solana/web3.js";
import { sign } from "tweetnacl";
import { TextEncoder } from "util";

export const genrateNonce = () => {
  const nonce = crypto.randomBytes(16).toString("hex");
  return nonce;
};

export const isValidNonce = (nonce: string) => {
  const buff = Buffer.from(nonce, "hex");
  if (buff.length !== 16) {
    return false;
  }
  return true;
};

export const verifyNonce = (
  nonce: string,
  signedNonce: string,
  pubKeyString: string
) => {
  return sign.detached.verify(
    new TextEncoder().encode(nonce),
    new TextEncoder().encode(signedNonce),
    new PublicKey(pubKeyString).toBytes()
  );
};
