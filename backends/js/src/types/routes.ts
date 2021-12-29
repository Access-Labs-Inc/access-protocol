import { JSONSchemaType } from "ajv";

/**
 * Bearer token
 */

export interface AuthorizationHeader {
  authorization: string;
}

export const AuthorizationHeaderSchema: JSONSchemaType<AuthorizationHeader> = {
  type: "object",
  properties: { authorization: { type: "string", format: "bearerToken" } },
  required: ["authorization"],
  additionalProperties: true,
};

/**
 * Auth
 */

export interface Payload {
  address: string;
  iat: number;
}

export const PayloadSchema: JSONSchemaType<Payload> = {
  type: "object",
  properties: { address: { type: "string" }, iat: { type: "number" } },
  required: ["address", "iat"],
  additionalProperties: false,
};

/**
 * POST /auth/nonce
 */

export interface NonceRequest {
  address: string;
}

export const NonceRequestSchema: JSONSchemaType<NonceRequest> = {
  type: "object",
  properties: { address: { type: "string", format: "pubkey" } },
  required: ["address"],
  additionalProperties: false,
};

export interface NonceResponse {
  nonce: string;
}

/**
 * POST /auth/login
 */

export interface LoginRequest {
  address: string;
  signedNonce: string;
}

export const LoginRequestSchema: JSONSchemaType<LoginRequest> = {
  type: "object",
  properties: {
    address: { type: "string", format: "pubkey" },
    signedNonce: { type: "string", format: "nonce" },
  },
  required: ["address", "signedNonce"],
  additionalProperties: false,
};

export interface LoginResponse {
  token: string;
}
