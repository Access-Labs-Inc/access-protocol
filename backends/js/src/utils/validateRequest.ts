import { Schema } from "ajv";
import { NextFunction, Response, Request } from "express";
import { ajv } from "./ajv";

/**
 * Express middle ware used to validate requests' bodies
 * @param schema The Ajv schema used to validate the request's body
 * @returns
 */
export const validaRequestBody =
  (schema: Schema) => (req: Request, res: Response, next: NextFunction) => {
    const validate = ajv.compile(schema);
    const isValid = validate(req.body);

    if (!isValid && req.method !== "GET") {
      return res.status(400).json(validate.errors);
    }
    next();
  };
