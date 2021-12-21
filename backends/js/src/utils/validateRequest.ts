import Ajv, { Schema } from "ajv";
import { NextFunction, Response, Request } from "express";
import { isValidPubkey } from "./pubkey";
import { isValidNonce } from "./nonce";

export const validaRequestBody =
  (schema: Schema) => (req: Request, res: Response, next: NextFunction) => {
    // https://ajv.js.org/options.html
    const ajv = new Ajv({ allErrors: true });

    // https://ajv.js.org/guide/formats.html#user-defined-formats
    ajv.addFormat("pubkey", {
      type: "string",
      validate: (x: string) => isValidPubkey(x),
    });
    ajv.addFormat("nonce", {
      type: "string",
      validate: (x: string) => isValidNonce(x),
    });

    const validate = ajv.compile(schema);
    const isValid = validate(req.body);

    if (!isValid && req.method !== "GET") {
      return res.status(400).json(validate.errors);
    }
    next();
  };
