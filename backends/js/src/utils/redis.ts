import { createClient } from "redis";

const EXPIRE_TIME = 10 * 60; // in seconds

export enum RedisKey {
  Nonce = "nonce:",
  Stake = "stake:",
}

export const redisClient = createClient(); // Can pass URL

export const setNonce = async (nonce: string, user: string) => {
  await redisClient.set(RedisKey.Nonce + user, nonce, { EX: EXPIRE_TIME });
};

export const getNonce = async (user: string) => {
  return await redisClient.get(RedisKey.Nonce + user);
};
