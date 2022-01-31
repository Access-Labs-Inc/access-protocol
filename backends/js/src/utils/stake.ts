import { StakeAccount, ACCESS_PROGRAM_ID } from "@access-protocol";
import { connection } from "./connection";
import { PublicKey } from "@solana/web3.js";

/**
 * Public key of the stake pool (must be defined in the .env file)
 */
const STAKE_POOL_KEY = new PublicKey(process.env.STAKE_POOL_KEY!);

/**
 * Minimum amount of tokens to stake to get access to the content
 * This information is stored in the stake pool header, hardcoding saves a few RPC calls
 */
const STAKE_MIN = parseInt(process.env.STAKE_MIN!);

/**
 * Verifies that a user has enough tokens staked
 * @param owner The owner of the stake account
 * @returns
 */
export const checkStake = async (owner: string) => {
  const [key] = await StakeAccount.getKey(
    ACCESS_PROGRAM_ID,
    new PublicKey(owner),
    STAKE_POOL_KEY
  );
  const stakeAccount = await StakeAccount.retrieve(connection, key);
  return stakeAccount.stakeAmount.toNumber() > STAKE_MIN;
};
