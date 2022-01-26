import { StakeAccount, ACCESS_PROGRAM_ID } from "@access-protocol";
import { connection } from "./connection";
import { PublicKey } from "@solana/web3.js";

const STAKE_POOL_KEY = new PublicKey(process.env.STAKE_POOL_KEY!);
const STAKE_MIN = parseInt(process.env.STAKE_MIN!);

export const checkStake = async (owner: string) => {
  const [key] = await StakeAccount.getKey(
    ACCESS_PROGRAM_ID,
    new PublicKey(owner),
    STAKE_POOL_KEY
  );
  const stakeAccount = await StakeAccount.retrieve(connection, key);
  return stakeAccount.stakeAmount.toNumber() > STAKE_MIN;
};
