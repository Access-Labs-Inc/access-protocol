import { StakeAccount, ACCESS_PROGRAM_ID, StakePool } from "@access-protocol";
import { connection } from "./connection";
import { PublicKey } from "@solana/web3.js";

/**
 * Public key of the stake pool (must be defined in the .env file)
 */
const STAKE_POOL_KEY = new PublicKey(process.env.STAKE_POOL_KEY!);

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
  const stakePool = await StakePool.retrieve(
    connection,
    stakeAccount.stakePool
  );

  const requiredAmount = Math.min(
    stakeAccount.poolMinimumAtCreation.toNumber(),
    stakePool.minimumStakeAmount.toNumber()
  );

  return stakeAccount.stakeAmount.toNumber() > requiredAmount;
};
