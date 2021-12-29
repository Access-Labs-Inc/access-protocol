import Ajv from "ajv";
import { isValidPubkey } from "./pubkey";
import { isValidNonce } from "./nonce";

export const ajv = new Ajv({ allErrors: true });

ajv.addFormat("bearerToken", {
  type: "string",
  validate: (x: string) => x.startsWith("Bearer ") && x.split(" ").length > 0,
});

ajv.addFormat("pubkey", {
  type: "string",
  validate: (x: string) => isValidPubkey(x),
});
ajv.addFormat("nonce", {
  type: "string",
  validate: (x: string) => isValidNonce(x),
});
