import { Connection, MemcmpFilter, PublicKey, TransactionInstruction } from "@solana/web3.js";
import { ACCESS_MINT, ACCESS_PROGRAM_ID, BondAccount, CentralStateV2, StakeAccount, StakePool } from "./state.js";
import * as BN from "bn.js";
import { claimRewards, crank, createStakeAccount, stake } from "./bindings";
import { createTransferInstruction, getAssociatedTokenAddressSync } from "@solana/spl-token";

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
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "C", // 12 in base 58 - todo test
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
  let lockedAmount: BN = new BN.BN(0);

  let stakeAccount: StakeAccount | undefined = undefined;
  try {
    stakeAccount = await StakeAccount.retrieve(connection, stakeKey);
    lockedAmount = lockedAmount.add(stakeAccount.stakeAmount);
  } catch (e) {
    console.error("Could not find lock account. Error: ", e);
  }

  // sum of Bond Accounts and BondV2 Accounts
  let bondsAmountSum = new BN.BN(0);

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

  const allBondV2AccountsForUser = await getBondV2Accounts(
    connection,
    pubkey,
    programId
  );
  if (allBondV2AccountsForUser != null && allBondV2AccountsForUser.length > 0) {
    allBondV2AccountsForUser.forEach((ba) => {
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

/**
 * This function can be used to get all instructions needed for a successful lock
 * @param connection The Solana RPC connection
 * @param user The user's pubkey
 * @param pool The pool's pubkey
 * @param feePayer The fee payer's pubkey
 * @param amount The amount to lock
 * @param feePayerCompensation The amount of ACS to reimburse to the fee payer if creating a StakeAccount
 * @param programId The program ID
 * @param centralState The central state, if already known (otherwise retrieved from the blockchain)
 * @param poolData The pool data, if already known (otherwise retrieved from the blockchain)
 */
export const fullLock = async (
  connection: Connection,
  user: PublicKey,
  pool: PublicKey,
  feePayer: PublicKey,
  amount: number,
  feePayerCompensation = 0,
  programId = ACCESS_PROGRAM_ID,
  centralState?: CentralStateV2,
  poolData?: StakePool,
): Promise<TransactionInstruction[]> => {
  const [stakeAccountPubkey] = StakeAccount.getKey(
    programId,
    user,
    pool,
  );

  const [centralStateKey] = CentralStateV2.getKey(programId);
  if (!centralState) {
    centralState = await CentralStateV2.retrieve(connection, centralStateKey);
  }
  if (!poolData) {
    poolData = await StakePool.retrieve(connection, pool);
  }
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = centralState.tokenMint;
  }

  const ixs: TransactionInstruction[] = [];

  let hasCranked = false;
  if (
    centralState.lastSnapshotOffset.toNumber() > poolData.currentDayIdx ||
    centralState.creationTime.toNumber() +
    86400 * (poolData.currentDayIdx + 1) <
    Date.now() / 1000
  ) {
    ixs.push(crank(pool, programId));
    hasCranked = true;
  }

  let stakeAccount = null;
  try {
    stakeAccount = await StakeAccount.retrieve(
      connection,
      stakeAccountPubkey,
    );
  } catch (err) {

    // Pay SOL for the account creation and reimburse in ACS
    if (feePayerCompensation > 0) {
      const from = user;
      const to = feePayer;
      const sourceATA = getAssociatedTokenAddressSync(tokenMint, from);
      const destinationATA = getAssociatedTokenAddressSync(tokenMint, to);
      ixs.push(createTransferInstruction(
        sourceATA,
        destinationATA,
        from,
        feePayerCompensation,
      ));
    }

    // Create stake account
    ixs.push(createStakeAccount(
      new PublicKey(pool),
      user,
      feePayer,
      programId,
    ));
  }

  if (
    stakeAccount &&
    stakeAccount.stakeAmount.toNumber() > 0 &&
    (stakeAccount.lastClaimedOffset.toNumber() < poolData.currentDayIdx ||
      hasCranked)
  ) {
    ixs.push(await claimRewards(
      connection,
      user,
      pool,
      programId,
    ));
  }

  ixs.push(await stake(
    connection,
    user,
    pool,
    amount * 10 ** 6,
    programId,
  ))

  return ixs;
}