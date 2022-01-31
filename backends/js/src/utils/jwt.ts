import { ajv } from "./ajv";
import { NextFunction, Response, Request } from "express";
import jwt from "jsonwebtoken";
import {
  Payload,
  AuthorizationHeaderSchema,
  AuthorizationHeader,
} from "../types/routes";

/**
 * Access token used to sign JWTs (must be defined in the .env file)
 */
export const ACCESS_TOKEN_SECRET = process.env.ACCESS_TOKEN_SECRET as string;

/**
 * TTL of JWTs (mut be defined in the .env file)
 */
const EXPIRATION_INTERVAL = parseFloat(process.env.EXPIRATION_INTERVAL!);

/**
 * Callback used in the JWT verification process.
 * If the signature verification fails the callback returns a 403.
 * If the signature is verified the callback verifies the time validty of the token.
 * If the token has expired the callback returns a 403
 * If all verifications are successful, the callback calls the next() Express function
 * @param res Express request
 * @param next Express next function
 * @returns
 */
const callback =
  (res: Response, next: NextFunction) =>
  (err: jwt.VerifyErrors | null, payload: jwt.JwtPayload | undefined) => {
    if (err) {
      console.error(err);
      return res.sendStatus(403);
    }
    const now = new Date().getTime();
    const { iat } = payload as Payload;
    if (now - iat > EXPIRATION_INTERVAL) {
      return res.sendStatus(403);
    }
    next();
  };

/**
 * Express middleware used to validate JWTs
 * @param req Express request
 * @param res Express response
 * @param next Express next function
 * @returns
 */
export const validateToken = (
  req: Request,
  res: Response,
  next: NextFunction
) => {
  const validate = ajv.compile(AuthorizationHeaderSchema);
  const isValid = validate(req.headers);

  if (!isValid) {
    return res.sendStatus(403);
  }

  const { authorization } = req.headers as AuthorizationHeader;
  const token = authorization.split(" ")[1];

  jwt.verify(token, ACCESS_TOKEN_SECRET, callback(res, next));
};
