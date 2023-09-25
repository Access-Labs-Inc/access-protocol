import { Connection, PublicKey, MemcmpFilter } from "@solana/web3.js";
import { StakeAccount, BondAccount, StakePool } from "./state.js";
import BN from "bn.js";

/**
 * This function can be used to find all stake accounts of a user
 * @param connection The Solana RPC connection
 * @param owner The owner of the stake accounts to retrieve
 * @param programId The program ID
 * @returns
 */
export const getStakeAccounts = async (
  connection: Connection,
  owner: PublicKey,
  programId: PublicKey
) => {
  const filters: MemcmpFilter[] = [
    {
      memcmp: {
        offset: 0,
        bytes: "4",
      },
    },
    {
      memcmp: {
        offset: 1,
        bytes: owner.toBase58(),
      },
    },
  ];
  return await connection.getProgramAccounts(programId, {
    filters,
  });
};

/**
 * This function can be used to find all stake pools of a user
 * @param connection The Solana RPC connection
 * @param owner The owner of the stake pools to retrieve
 * @param programId The program ID
 * @returns
 */
export const getStakePools = async (
  connection: Connection,
  owner: PublicKey,
  programId: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "2",
      },
    },
    {
      memcmp: {
        offset: 1 + 1 + 2 + 4 + 8 + 8 + 8,
        bytes: owner.toBase58(),
      },
    },
  ];
  return await connection.getProgramAccounts(programId, {
    filters,
  });
};

/**
 * This function can be used to find all bonds of a user
 * @param connection The Solana RPC connection
 * @param owner The owner of the bonds to retrieve
 * @param programId The program ID
 * @returns
 */
export const getBondAccounts = async (
  connection: Connection,
  owner: PublicKey,
  programId: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "6",
      },
    },
    {
      memcmp: {
        offset: 1,
        bytes: owner.toBase58(),
      },
    },
  ];
  return await connection.getProgramAccounts(programId, {
    filters,
  });
};


/**
 * This function can be used to find all bondV2s of a user
 * @param connection The Solana RPC connection
 * @param owner The owner of the bonds to retrieve
 * @param programId The program ID
 * @returns
 */
export const getBondV2Accounts = async (
  connection: Connection,
  owner: PublicKey,
  programId: PublicKey
) => {
  // "12" as base58
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "12", // todo test and possibly fix
      },
    },
    {
      memcmp: {
        offset: 1,
        bytes: owner.toBase58(),
      },
    },
  ];
  return await connection.getProgramAccounts(programId, {
    filters,
  });
};

/**
 * This function can be used to retrieve all the stake pools
 * @param connection The Solana RPC connection
 * @param programId The program ID
 * @returns
 */
export const getAllStakePools = async (
  connection: Connection,
  programId: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "2",
      },
    },
  ];
  return await connection.getProgramAccounts(programId, {
    filters,
  });
};

/**
 * This function can be used to retrieve all the inactive stake pools
 * @param connection The Solana RPC connection
 * @param programId The program ID
 * @returns
 */
export const getAllInactiveStakePools = async (
  connection: Connection,
  programId: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "3",
      },
    },
  ];
  return await connection.getProgramAccounts(programId, {
    filters,
  });
};

/**
 * This function can be used to retrieve all the inactive bonds
 * @param connection The Solana RPC connection
 * @param programId The program ID
 * @returns
 */
export const getAllInactiveBonds = async (
  connection: Connection,
  programId: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "5",
      },
    },
  ];
  return await connection.getProgramAccounts(programId, {
    filters,
  });
};

/**
 * This function can be used to retrieve all the active bonds
 * @param connection The Solana RPC connection
 * @param programId The program ID
 * @returns
 */
export const getAllActiveBonds = async (
  connection: Connection,
  programId: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "6",
      },
    },
  ];
  return await connection.getProgramAccounts(programId, {
    filters,
  });
};

/**
 * This function can be used to get locked amount for specific pool (stake + bonds)
 * @param connection The Solana RPC connection
 * @param programId The program ID
 * @param poolPubkey Public key of pool
 * @param pubkey User's pubkey
 * @returns BN
 */
export const getLockedAmountForPool = async (
  connection: Connection,
  programId: PublicKey,
  poolPubkey: PublicKey,
  pubkey: PublicKey
): Promise<BN> => {
  const [stakeKey] = await StakeAccount.getKey(programId, pubkey, poolPubkey);

  // SUM of locked tokens (aka Stake Account)
  let lockedAmount: BN = new BN(0);

  let stakeAccount: StakeAccount | undefined = undefined;
  try {
    stakeAccount = await StakeAccount.retrieve(connection, stakeKey);
    lockedAmount = lockedAmount.add(stakeAccount.stakeAmount);
  } catch (e) {
    console.error("Could not find lock account. Error: ", e);
  }

  // SUM of airdrop tokens (aka Bond Accounts)
  let bondsAmountSum = new BN(0);

  const allBondAccountsForUser = await getBondAccounts(
    connection,
    pubkey,
    programId
  );
  if (allBondAccountsForUser != null && allBondAccountsForUser.length > 0) {
    allBondAccountsForUser.forEach((ba) => {
      const b = BondAccount.deserialize(ba.account.data);
      if (b.stakePool.toBase58() === poolPubkey.toBase58()) {
        bondsAmountSum = bondsAmountSum.add(b.totalStaked);
      }
    });
  }

  return lockedAmount.add(bondsAmountSum);
};

/**
 * This function can be used to get locked amount for specific pool (stake + bonds)
 * @param connection The Solana RPC connection
 * @param programId The program ID
 * @param poolPubkey Public key of pool
 * @param pubkey User's pubkey
 * @returns Boolean
 */
export const hasValidSubscriptionForPool = async (
  connection: Connection,
  programId: PublicKey,
  poolPubkey: PublicKey,
  pubkey: PublicKey
): Promise<Boolean> => {
  let poolAccount: StakePool | undefined = undefined;
  try {
    poolAccount = await StakePool.retrieve(connection, poolPubkey);
  } catch (e) {
    console.error("Could not find stake pool account. Error: ", e);
    return false;
  }

  const [stakeKey] = await StakeAccount.getKey(programId, pubkey, poolPubkey);

  let stakeAccount: StakeAccount | undefined = undefined;
  try {
    stakeAccount = await StakeAccount.retrieve(connection, stakeKey);
  } catch (e) {
    console.error("Could not find lock account. Error: ", e);
  }

  const requiredMinAmountToLock = stakeAccount
    ? Math.min(
        Number(stakeAccount.poolMinimumAtCreation),
        Number(poolAccount.minimumStakeAmount)
      )
    : Number(poolAccount.minimumStakeAmount);

  const lockedAmount = await getLockedAmountForPool(
    connection,
    programId,
    poolPubkey,
    pubkey
  );
  return lockedAmount.toNumber() >= requiredMinAmountToLock;
};
