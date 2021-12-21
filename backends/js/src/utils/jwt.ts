import Ajv, { Schema } from "ajv";
import { NextFunction, Response, Request } from "express";
import jwt from "jsonwebtoken";
import {
  Payload,
  AuthorizationHeaderSchema,
  AuthorizationHeader,
} from "../types/routes";

export const ACCESS_TOKEN_SECRET = process.env.ACCESS_TOKEN_SECRET as string;
export const REFRESH_TOKEN_SECRET = process.env.REFRESH_TOKEN_SECRET as string;

const EXPIRATION_INTERVAL = 24 * 60 * 60 * 1000;

const ajv = new Ajv({ allErrors: true });

ajv.addFormat("bearerToken", {
  type: "string",
  validate: (x: string) => x.startsWith("Bearer ") && x.split(" ").length > 0,
});

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
