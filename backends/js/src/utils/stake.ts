import { StakeAccount, ACCESS_PROGRAM_ID, StakePool, BondAccount, getBondAccounts, getAllActiveBonds, getAllInactiveBonds } from "@access-protocol";
import { connection } from "./connection";
import { PublicKey } from "@solana/web3.js";

/**
 * Public key of the stake pool (must be defined in the .env file)
 */
const STAKE_POOL_KEY = new PublicKey(process.env.STAKE_POOL_KEY!);
const BOND_POOL_KEY = process.env.BOND_POOL_KEY!;

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

  const bondAccounts = await getBondAccounts(connection, new PublicKey(owner));
  let bondTotalStaked = 0;
  for (const bondAccount of bondAccounts) {
    const account = BondAccount.deserialize(bondAccount.account.data);
    bondTotalStaked += account.totalStaked.toNumber();
  }

  const requiredAmount = Math.min(
    stakeAccount.poolMinimumAtCreation.toNumber(),
    stakePool.minimumStakeAmount.toNumber()
  );

  return stakeAccount.stakeAmount.toNumber() + bondTotalStaked > requiredAmount;
};
