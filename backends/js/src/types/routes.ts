import { JSONSchemaType } from "ajv";

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
  nonce: string;
}

export const LoginRequestSchema: JSONSchemaType<LoginRequest> = {
  type: "object",
  properties: {
    address: { type: "string", format: "pubkey" },
    nonce: { type: "string", format: "nonce" },
  },
  required: ["address", "nonce"],
  additionalProperties: false,
};
