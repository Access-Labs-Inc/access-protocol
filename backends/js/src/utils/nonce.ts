import crypto from "crypto";

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
