import { createClient } from "redis";

/**
 * TTL of nonces in redis (mut be defined in the .env file)
 */
const EXPIRE_TIME = parseFloat(process.env.REDIS_EXPIRE_TIME!); // in seconds

/**
 * Redis keys prefix
 */
export enum RedisKey {
  Nonce = "nonce:",
  Stake = "stake:",
}

/**
 * Redis client
 */
export const redisClient = createClient(); // Change this in production

/**
 * Stores a nonce in redis with a TTL of EXPIRE_TIME
 * @param nonce Nonce to store in redis
 * @param user Public key of the user to which the nonce is assigned
 */
export const setNonce = async (nonce: string, user: string) => {
  await redisClient.set(RedisKey.Nonce + user, nonce, { EX: EXPIRE_TIME });
};

/**
 * Returns the latest nonce generated for a user
 * @param user Public key of the user
 * @returns
 */
export const getNonce = async (user: string) => {
  return await redisClient.get(RedisKey.Nonce + user);
};
