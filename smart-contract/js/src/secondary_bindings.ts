import { Connection, MemcmpFilter, PublicKey, TransactionInstruction } from "@solana/web3.js";
import {
  ACCESS_MINT,
  ACCESS_NFT_PROGRAM_SIGNER,
  ACCESS_PROGRAM_ID,
  BondAccount,
  BondV2Account,
  CentralStateV2,
  RoyaltyAccount,
  StakeAccount,
  StakePool,
  Tag
} from "./state.js";
import * as BN from "bn.js";
import {
  addToBondV2,
  claimBondV2Rewards,
  claimRewards,
  crank,
  createBondV2,
  createStakeAccount,
  stake,
  unstake
} from "./bindings.js";
import {
  createAssociatedTokenAccountInstruction,
  createTransferInstruction,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {
  claimBondRewardsInstruction,
  claimBondV2RewardsInstruction,
  claimRewardsInstruction
} from "./raw_instructions.js";

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
 * This function can be used to retrieve the Royalty accounts of all people paying royalties to the recipient
 * @param connection The Solana RPC connection
 * @param recipient The recipient's public key
 * @param currentTimestamp The current Solana timestamp
 * @param programId The program ID
 * @returns
 */
export const getActiveRoyaltyPayers = async (
  connection: Connection,
  recipient: PublicKey,
  currentTimestamp: number,
  programId: PublicKey
) => {
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    const centralStateKey = CentralStateV2.getKey(programId)[0];
    const centralState = await CentralStateV2.retrieve(connection, centralStateKey);
    tokenMint = centralState.tokenMint;
  }
  const ata = getAssociatedTokenAddressSync(tokenMint, recipient);
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "E",
      },
    },
    {
      memcmp: {
        offset: 1 + 2* 32,
        bytes: ata.toBase58(),
      },
    },
  ];
  return (await connection.getProgramAccounts(programId, {
    filters,
  })).map(e => RoyaltyAccount.deserialize(e.account.data))
    .filter(e =>
      e.expirationDate.toNumber() > currentTimestamp
    );
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
 * @param amount The amount to lock in ACS
 * @param currentTimestamp The current Solana timestamp
 * @param feePayerCompensation The amount of ACS to reimburse to the fee payer if creating a StakeAccount
 * @param programId The program ID
 * @param centralState The central state, if already known (otherwise retrieved from the blockchain)
 * @param poolData The pool data, if already known (otherwise retrieved from the blockchain)
 * @param unlockDate The unlock date for the purpose of locking tokens in a bond. Special values are:
 * -1: Not a bond account (default, unlocks immediately)
 * 0: Forever bond (no unlock date)
 */
export const fullLock = async (
  connection: Connection,
  user: PublicKey,
  pool: PublicKey,
  feePayer: PublicKey,
  amount: number,
  currentTimestamp: number,
  feePayerCompensation = 0,
  programId = ACCESS_PROGRAM_ID,
  centralState?: CentralStateV2,
  poolData?: StakePool,
  unlockDate = -1
): Promise<TransactionInstruction[]> => {

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
    86400 * (centralState.lastSnapshotOffset.toNumber() + 1) < currentTimestamp
  ) {
    ixs.push(crank(pool, programId));
    hasCranked = true;
  }

  if (unlockDate === -1) {
    ixs.push(...(await lockStakeAccount(
      connection,
      user,
      pool,
      feePayer,
      amount,
      feePayerCompensation,
      programId,
      tokenMint,
      poolData,
      hasCranked,
    )))
  } else {
    ixs.push(...(await lockBondV2Account(
      connection,
      user,
      pool,
      feePayer,
      amount,
      feePayerCompensation,
      programId,
      tokenMint,
      poolData,
      hasCranked,
      unlockDate,
    )))
  }

  return ixs;
}

const lockStakeAccount = async (
  connection: Connection,
  user: PublicKey,
  pool: PublicKey,
  feePayer: PublicKey,
  amount: number,
  feePayerCompensation = 0,
  programId: PublicKey,
  tokenMint: PublicKey,
  poolData: StakePool,
  hasCranked: boolean,
) => {
  const ixs = [];
  let stakeAccount = null;
  try {
    stakeAccount = await StakeAccount.retrieve(
      connection,
      StakeAccount.getKey(
        programId,
        user,
        pool,
      )[0],
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
      pool,
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

const lockBondV2Account = async (
  connection: Connection,
  user: PublicKey,
  pool: PublicKey,
  feePayer: PublicKey,
  amount: number,
  feePayerCompensation = 0,
  programId: PublicKey,
  tokenMint: PublicKey,
  poolData: StakePool,
  hasCranked: boolean,
  unlockDate: number,
) => {
  const ixs = [];

  let bondV2Account = null;
  const bondV2AccountKey = BondV2Account.getKey(
    programId,
    user,
    pool,
    new BN.BN(unlockDate),
  )[0];
  try {
    bondV2Account = await BondV2Account.retrieve(
      connection,
      bondV2AccountKey
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

    // Create bondV2 account
    ixs.push(createBondV2(
      user,
      feePayer,
      pool,
      unlockDate ? new BN.BN(unlockDate) : null,
      programId,
    ));
  }

  if (
    bondV2Account &&
    bondV2Account.amount.toNumber() > 0 &&
    (bondV2Account.lastClaimedOffset.toNumber() < poolData.currentDayIdx ||
      hasCranked)
  ) {
    ixs.push(await claimBondV2Rewards(
      connection,
      bondV2AccountKey,
      programId,
    ));
  }

  ixs.push(await addToBondV2(
    connection,
    user,
    user,
    pool,
    new BN.BN(amount).mul(new BN.BN(10 ** 6)),
    new BN.BN(unlockDate),
    programId,
  ));

  return ixs;
}


/**
 * This function can be used to get all instructions needed for a successful unlock
 * @param connection The Solana RPC connection
 * @param user The user's pubkey
 * @param pool The pool's pubkey
 * @param amount The amount to unlock in ACS
 * @param currentTimestamp The current Solana timestamp
 * @param programId The program ID
 * @param centralState The central state, if already known (otherwise retrieved from the blockchain)
 * @param poolData The pool data, if already known (otherwise retrieved from the blockchain)
 * @param stakeAccount The stake account, if already known (otherwise retrieved from the blockchain)
 */
export const fullUnlock = async (
  connection: Connection,
  user: PublicKey,
  pool: PublicKey,
  amount: number,
  currentTimestamp: number,
  programId = ACCESS_PROGRAM_ID,
  centralState?: CentralStateV2,
  poolData?: StakePool,
  stakeAccount?: StakeAccount,
): Promise<TransactionInstruction[]> => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  if (!centralState) {
    centralState = await CentralStateV2.retrieve(connection, centralStateKey);
  }
  if (!poolData) {
    poolData = await StakePool.retrieve(connection, pool);
  }
  if (!stakeAccount) {
    stakeAccount = await StakeAccount.retrieve(
      connection,
      StakeAccount.getKey(
        programId,
        user,
        pool,
      )[0],
    );
  }

  const ixs: TransactionInstruction[] = [];
  let hasCranked = false;
  if (
    centralState.lastSnapshotOffset.toNumber() > poolData.currentDayIdx ||
    centralState.creationTime.toNumber() +
    86400 * (poolData.currentDayIdx + 1) <
    currentTimestamp
  ) {
    ixs.push(crank(pool, programId));
    hasCranked = true;
  }

  if (
    stakeAccount.stakeAmount.toNumber() > 0 &&
    (stakeAccount.lastClaimedOffset.toNumber() < poolData.currentDayIdx ||
      hasCranked)
  ) {
    ixs.push(
      await claimRewards(
        connection,
        user,
        pool,
        programId,
      ),
    );
  }

  if (
    stakeAccount.stakeAmount.toNumber() > 0 &&
    amount > 0
  ) {
    ixs.push(
      await unstake(
        connection,
        user,
        pool,
        amount * 10 ** 6,
        programId,
      ),
    );
  }

  return ixs;
}

/** This function can be used to get all instructions needed for a successful claim
 * The instructions are returned in two arrays, first one is a set of instructions that need to be called before the second one
 * @param connection The Solana RPC connection
 * @param user The user's pubkey
 * @param feePayer The fee payer's pubkey
 * @param feePayerCompensation The amount of ACS to reimburse to the fee payer
 * @param programId The program ID
 * @param poolOffsets The pool offsets, if already known (otherwise retrieved from the blockchain)
 * @param tokenAccountFeepayer The token account of the fee payer
 */
export const fullUserRewardClaim = async (
  connection: Connection,
  user: PublicKey,
  feePayer: PublicKey,
  feePayerCompensation = 0,
  programId = ACCESS_PROGRAM_ID,
  poolOffsets: Map<string, number> | undefined = undefined,
  tokenAccountFeepayer = feePayer,
): Promise<[TransactionInstruction[], TransactionInstruction[]]> => {
  const filters: MemcmpFilter[] = [
    {
      memcmp: {
        offset: 1,
        bytes: user.toBase58(),
      },
    },
  ];
  const userOwnerAccounts = await connection.getProgramAccounts(programId, {
    filters,
  });
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(
    connection,
    centralStateKey,
  );
  const currentOffset = await centralState.getCurrentOffset(connection);

  const userATA = getAssociatedTokenAddressSync(centralState.tokenMint, user);
  const relevantPools = new Set<string>();
  const rewardsDestination = getAssociatedTokenAddressSync(
    centralState.tokenMint,
    user,
    true,
  );
  const [royaltyAccountAddr] = RoyaltyAccount.getKey(programId, user);
  const ownerRoyaltyAccount = await RoyaltyAccount.retrieve(connection, royaltyAccountAddr);
  const royaltyAta = ownerRoyaltyAccount ? ownerRoyaltyAccount.recipientAta : null;

  const claimIxs = userOwnerAccounts
    .map(account => {
      switch (account.account.data[0]) {
        // stake account
        case Tag.StakeAccount:
          const stakeAccount = StakeAccount.deserialize(account.account.data);
          if (stakeAccount.stakeAmount.toNumber() === 0) {
            return null;
          }
          relevantPools.add(stakeAccount.stakePool.toBase58());
          if (stakeAccount.lastClaimedOffset < currentOffset) {
            const claimIx = new claimRewardsInstruction({
              allowZeroRewards: Number(false),
            }).getInstruction(
              programId,
              stakeAccount.stakePool,
              account.pubkey,
              user,
              rewardsDestination,
              centralStateKey,
              centralState.tokenMint,
              ACCESS_NFT_PROGRAM_SIGNER,
              TOKEN_PROGRAM_ID,
              royaltyAccountAddr,
              royaltyAta
            );

            return claimIx;
          }
          return null;

        // bond account
        case Tag.BondAccount:
          const bondAccount = BondAccount.deserialize(account.account.data);
          if (bondAccount.totalStaked.toNumber() === 0) {
            return null;
          }
          relevantPools.add(bondAccount.stakePool.toBase58());
          if (bondAccount.lastClaimedOffset < currentOffset) {
            const bondClaimIx =
              new claimBondRewardsInstruction().getInstruction(
                programId,
                bondAccount.stakePool,
                account.pubkey,
                bondAccount.owner,
                rewardsDestination,
                centralStateKey,
                centralState.tokenMint,
                TOKEN_PROGRAM_ID,
              );
            return bondClaimIx;
          }
          return null;

        // bondV2 account
        case Tag.BondV2Account:
          const bondV2Account = BondV2Account.deserialize(account.account.data);
          if (bondV2Account.amount.toNumber() === 0) {
            return null;
          }
          relevantPools.add(bondV2Account.pool.toBase58());
          if (bondV2Account.lastClaimedOffset < currentOffset) {
            return new claimBondV2RewardsInstruction().getInstruction(
              programId,
              bondV2Account.pool,
              account.pubkey,
              bondV2Account.owner,
              rewardsDestination,
              centralStateKey,
              centralState.tokenMint,
              ACCESS_NFT_PROGRAM_SIGNER,
              TOKEN_PROGRAM_ID,
              royaltyAccountAddr,
              royaltyAta
            );
          }
          return null;
        default:
          return null;
      }
    })
    .filter(e => e !== null) as TransactionInstruction[];

  const preClaimIxs = [];
  const userATAInfo = await connection.getAccountInfo(userATA);
  if (!userATAInfo) {
    preClaimIxs.push(
      createAssociatedTokenAccountInstruction(
        tokenAccountFeepayer,
        userATA,
        user,
        centralState.tokenMint,
      ),
    );
    if (feePayerCompensation > 0) {
      const from = user;
      const to = feePayer;
      const sourceATA = getAssociatedTokenAddressSync(
        centralState.tokenMint,
        from,
      );
      const destinationATA = getAssociatedTokenAddressSync(
        centralState.tokenMint,
        to,
      );
      preClaimIxs.push(
        createTransferInstruction(
          sourceATA,
          destinationATA,
          from,
          feePayerCompensation,
        ),
      );
    }
  }

  let filledPoolOffsets = poolOffsets;
  if (!poolOffsets) {
    filledPoolOffsets = (await getAllStakePools(connection, programId)).reduce(
      (acc, poolData) => {
        const pool = StakePool.deserialize(poolData.account.data);
        acc.set(poolData.pubkey.toBase58(), pool.currentDayIdx);
        return acc;
      },
      new Map<string, number>(),
    );
  }
  if (!filledPoolOffsets) {
    throw new Error('Pool offsets not found');
  }

  relevantPools.forEach(poolAddress => {
    const poolOffset = filledPoolOffsets!.get(poolAddress) ?? null;
    if (poolOffset === null) {
      throw new Error('Pool offset not found');
    }
    if (poolOffset < currentOffset.toNumber()) {
      preClaimIxs.push(crank(new PublicKey(poolAddress), programId));
    }
  });

  return [preClaimIxs, claimIxs];
};