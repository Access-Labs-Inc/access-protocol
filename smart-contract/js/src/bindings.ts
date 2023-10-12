import {
  activateStakePoolInstruction,
  addToBondV2Instruction, adminChangeFreezeAuthorityInstruction,
  adminProgramFreezeInstruction,
  adminRenounceInstruction,
  adminSetProtocolFeeInstruction,
  adminSetupFeeSplitInstruction,
  changeCentralStateAuthorityInstruction,
  changeInflationInstruction,
  changePoolMinimumInstruction,
  changePoolMultiplierInstruction,
  claimBondRewardsInstruction,
  claimBondV2RewardsInstruction,
  claimPoolRewardsInstruction,
  claimRewardsInstruction,
  crankInstruction,
  createBondV2Instruction,
  createCentralStateInstruction,
  createStakeAccountInstruction,
  createStakePoolInstruction,
  distributeFeesInstruction,
  migrateCentralStateV2Instruction,
  stakeInstruction,
  TaggedInstruction,
  unlockBondTokensInstruction,
  unlockBondV2Instruction,
  unstakeInstruction,
} from "./raw_instructions.js";
import { Connection, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  ACCESS_MINT,
  ACCESS_PROGRAM_ID,
  BondAccount,
  BondV2Account,
  CentralState,
  CentralStateV2,
  FeeRecipient,
  StakeAccount,
  StakePool
} from "./state.js";
import * as BN from "bn.js";
import {
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

/**
 * This function can be used to update the inflation of the central state
 * @param connection The Solana RPC connection
 * @param newInflation The new inflation amount (in micro ACS tokens per day)
 * @param programId The ACCESS program ID
 * @returns ix The instruction to change the inflation
 */
export const adminChangeInflation = async (
  connection: Connection,
  newInflation: BN,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);

  return new changeInflationInstruction({
    dailyInflation: newInflation,
  }).getInstruction(programId, centralStateKey, centralState.authority, centralState.tokenMint);
};

/**
 * This function can be used to update the minimum amount of tokens that give access to the content
 * @param connection The Solana RPC connection
 * @param stakePoolKey The key of the stake pool
 * @param newMinimum The new minimum amount of tokens to stake to get access
 * @param programId The ACCESS program ID
 * @returns ix The instruction to change the pool minimum
 */
export const changePoolMinimum = async (
  connection: Connection,
  stakePoolKey: PublicKey,
  newMinimum: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stakePool = await StakePool.retrieve(connection, stakePoolKey);
  let [centralStateKey] = CentralStateV2.getKey(programId);

  return new changePoolMinimumInstruction({
    newMinimum: new BN.BN(newMinimum),
  }).getInstruction(programId, stakePoolKey, stakePool.owner, centralStateKey);
};

/**
 * This function can be used activate a created stake pool
 * @param stakePoolKey The key of the stake pool
 * @param programId The ACCESS program ID
 * @returns ix The instruction to activate the stake pool
 */
export const activateStakePool = (
  stakePoolKey: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  return new activateStakePoolInstruction().getInstruction(
    programId,
    stakePoolKey,
    centralStateKey
  );
};

/**
 * This function can be used by a bond owner to claim his staking rewards
 * @param connection The Solana RPC connection
 * @param bondAccount The key of the bond account
 * @param rewardsDestination The destination token account for the rewards being claimed
 * @param programId The ACCESS program ID
 * @param ownerMustSign todo
 * @returns ix The instruction to claim the bond rewards
 */
export const claimBondRewards = async (
  connection: Connection,
  bondAccount: PublicKey,
  rewardsDestination: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const bond = await BondAccount.retrieve(connection, bondAccount);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }

  const ix = new claimBondRewardsInstruction().getInstruction(
    programId,
    bond.stakePool,
    bondAccount,
    bond.owner,
    rewardsDestination,
    centralStateKey,
    tokenMint,
    TOKEN_PROGRAM_ID
  );

  // we don't require the owner to sign this transaction as our use-case is only the bond owner claiming their rewards.
  const idx = ix.keys.findIndex((e) => e.pubkey.equals(bond.owner));
  ix.keys[idx].isSigner = false;

  return ix;
};

/**
 * This function can be used by a pool owner to claim his staking rewards
 * @param connection The Solana RPC connection
 * @param stakePoolAccount The key of the stake pool
 * @param programId The ACCESS program ID
 * @returns ix The instruction to claim the pool rewards
 */
export const claimPoolRewards = async (
  connection: Connection,
  stakePoolAccount: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const stakePool = await StakePool.retrieve(connection, stakePoolAccount);
  const rewardsDestination = getAssociatedTokenAddressSync(
    tokenMint,
    stakePool.owner,
  );

  const ix = new claimPoolRewardsInstruction().getInstruction(
    programId,
    stakePoolAccount,
    stakePool.owner,
    rewardsDestination,
    centralStateKey,
    tokenMint,
    TOKEN_PROGRAM_ID
  );

  // we don't require the owner to sign this transaction as our use-case is only the pool owner claiming their rewards
  const idx = ix.keys.findIndex((e) => e.pubkey.equals(stakePool.owner));
  ix.keys[idx].isSigner = false;

  return ix;
};

/**
 * This function can be used by a supporter to claim their staking rewards
 * @param connection The Solana RPC connection
 * @param user The key of the user
 * @param pool The key of the stake pool
 * @param programId The ACCESS program ID
 * @returns ix The instruction to claim the rewards
 */
export const claimRewards = async (
  connection: Connection,
  user: PublicKey,
  pool: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [stakeAccount] = StakeAccount.getKey(programId, user, pool);
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const rewardsDestination = getAssociatedTokenAddressSync(
    tokenMint,
    user,
    true,
  );

  const ix = new claimRewardsInstruction({
    allowZeroRewards: Number(false),
  }).getInstruction(
    programId,
    pool,
    stakeAccount,
    user,
    rewardsDestination,
    centralStateKey,
    tokenMint,
    TOKEN_PROGRAM_ID
  );

  // we don't require the owner to sign this transaction as users are claiming for themselves
  const idx = ix.keys.findIndex((e) => e.pubkey.equals(user));
  ix.keys[idx].isSigner = false;

  return ix;
};

/**
 * This function can be used to calculate the rewaards for a stake pool.
 * It has to be called at least once per day for the rewards not to be discarded.
 * @param stakePoolAccount The key fo the stake pool to crank
 * @param programId The ACCESS program ID
 * @returns ix The instruction to crank the stake pool
 */
export const crank = (
  stakePoolAccount: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  return new crankInstruction().getInstruction(
    programId,
    stakePoolAccount,
    centralStateKey
  );
};

/**
 * This function is used to create the central state after deploying the program
 * @param dailyInflation The daily inflation (i.e. raw token amounts being emitted per day in micro ACS)
 * @param authority The central state authority (only key that will be able to perform admin operations)
 * @param mint The ACS token mint
 * @param programId The ACCESS program ID
 * @returns ix The instruction to create the central state
 */
export const createCentralState = async (
  dailyInflation: number,
  authority: PublicKey,
  mint: PublicKey,
  programId: PublicKey,
) => {
  const [centralStateKey] = CentralState.getKey(programId);

  return new createCentralStateInstruction({
    dailyInflation: new BN.BN(dailyInflation),
    authority: authority.toBuffer(),
  }).getInstruction(
    programId,
    centralStateKey,
    SystemProgram.programId,
    authority,
    mint
  );
};

/**
 * This function can be used to create a stake account in stake pool
 * @param stakePool The key of the stake pool
 * @param owner The owner of the staking account being created (i.e the staker)
 * @param feePayer The fee payer of the transaction
 * @param programId The ACCESS program ID
 * @returns
 */
export const createStakeAccount = (
  stakePool: PublicKey,
  owner: PublicKey,
  feePayer: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [stakeAccount, bumpSeed] = StakeAccount.getKey(
    programId,
    owner,
    stakePool
  );
  const [centralStateKey] = CentralStateV2.getKey(programId);

  return new createStakeAccountInstruction({
    nonce: bumpSeed,
    owner: owner.toBuffer(),
  }).getInstruction(
    programId,
    stakeAccount,
    SystemProgram.programId,
    stakePool,
    feePayer,
    centralStateKey
  );
};

/**
 * This instruction can be used by content creators to create a pool for their subscribers.
 * @param connection The Solana RPC connection
 * @param owner The owner of the stake pool (only key authorized to perform pool admin operations)
 * @param minimumStakeAmount The minimum amount of tokens to lock in the pool
 * @param feePayer The fee payer of the tx
 * @param programId The ACCESS program ID
 * @returns ix The instruction to create the stake pool
 */
export const createStakePool = async (
  connection: Connection,
  owner: PublicKey,
  minimumStakeAmount: number,
  feePayer: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [stakePool] = StakePool.getKey(programId, owner);
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const vault = getAssociatedTokenAddressSync(
    tokenMint,
    stakePool,
    true,
  );

  const createVaultIx = createAssociatedTokenAccountInstruction(
    feePayer,
    vault,
    stakePool,
    tokenMint,
  );

  const ix = new createStakePoolInstruction({
    owner: owner.toBuffer(),
    minimumStakeAmount: new BN.BN(minimumStakeAmount),
  }).getInstruction(
    programId,
    stakePool,
    SystemProgram.programId,
    feePayer,
    vault,
    centralStateKey,
  );

  return [createVaultIx, ix];
};

/**
 * This instruction can be used by supporters to deposit ACS tokens into their stake account.
 * The protocol fee will be deducted additionally to the `amount` from the source account.
 * @param connection The Solana RPC connection
 * @param user The key of the user
 * @param pool The key of the stake pool
 * @param amount The amount of tokens to stake
 * @param programId The ACCESS program ID
 * @returns
 */
export const stake = async (
  connection: Connection,
  user: PublicKey,
  pool: PublicKey,
  amount: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [stakeAccount] = StakeAccount.getKey(programId, user, pool);
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }

  const feesAta = getAssociatedTokenAddressSync(
    tokenMint,
    centralStateKey,
    true,
  );

  const poolVault = getAssociatedTokenAddressSync(
    tokenMint,
    pool,
    true,
  );

  const userAta = getAssociatedTokenAddressSync(
    tokenMint,
    user,
    true,
  );

  return new stakeInstruction({
    amount: new BN.BN(amount),
  }).getInstruction(
    programId,
    centralStateKey,
    stakeAccount,
    pool,
    user,
    userAta,
    TOKEN_PROGRAM_ID,
    poolVault,
    feesAta,
  );
};

/**
 * This instruction can be used by a bond owned to unlock ACCESS tokens. Once unlocked the tokens are not staked anymore.
 * @param connection The Solana RPC connection
 * @param bondAccount The key of the bond account
 * @param destinationToken The token account receiving the tokens
 * @param programId The ACCESS program ID
 * @returns
 */
export const unlockBondTokens = async (
  connection: Connection,
  bondAccount: PublicKey,
  destinationToken: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const bond = await BondAccount.retrieve(connection, bondAccount);
  const stakePool = await StakePool.retrieve(connection, bond.stakePool);

  return new unlockBondTokensInstruction().getInstruction(
    programId,
    bondAccount,
    bond.owner,
    tokenMint,
    destinationToken,
    centralStateKey,
    bond.stakePool,
    stakePool.vault,
    TOKEN_PROGRAM_ID
  );
};

/**
 * This instruction can be used to unlock ACS tokens from a pool
 * @param connection The Solana RPC connection
 * @param user The key of the user
 * @param pool The key of the stake pool
 * @param amount The amount of tokens to unlock
 * @param programId The ACCESS program ID
 * @returns
 */
export const unstake = async (
  connection: Connection,
  user: PublicKey,
  pool: PublicKey,
  amount: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [stakeAccountKey] = StakeAccount.getKey(programId, user, pool);
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const stakePoolVault = getAssociatedTokenAddressSync(tokenMint, pool, true);
  const destinationAccount = getAssociatedTokenAddressSync(
    tokenMint,
    user,
    true,
  );


  return new unstakeInstruction({
    amount: new BN.BN(amount),
  }).getInstruction(
    programId,
    centralStateKey,
    stakeAccountKey,
    pool,
    user,
    destinationAccount,
    TOKEN_PROGRAM_ID,
    stakePoolVault,
  );
};

/**
 * This function allows a pool owner to adjust the percentage of the pool rewards that go to the supporters.
 * @param connection The Solana RPC connection
 * @param stakePoolKey The key of the stake pool
 * @param newMultiplier The new multiplier (in percent [0-100]). This is the percentage of the pools rewards that go to the supporters.
 * @param programId The ACCESS program ID
 * @returns ix The instruction to change the pool multiplier
 */
export const changePoolMultiplier = async (
  connection: Connection,
  stakePoolKey: PublicKey,
  newMultiplier: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stakePool = await StakePool.retrieve(connection, stakePoolKey);
  const [centralStateKey] = CentralStateV2.getKey(programId);

  return new changePoolMultiplierInstruction({
    newMultiplier: new BN.BN(newMultiplier),
  }).getInstruction(programId, stakePoolKey, stakePool.owner, centralStateKey);
};

/**
 * This function can be used to change the central state authority
 * @param connection The Solana RPC connection
 * @param newAuthority The new authority of the central state
 * @param programId The ACCESS program ID
 * @returns ix The instruction to change the central state authority
 */
export const adminChangeCentralStateAuthority = async (
  connection: Connection,
  newAuthority: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);

  return new changeCentralStateAuthorityInstruction({
    newAuthority: newAuthority.toBytes(),
  }).getInstruction(programId, centralStateKey, centralState.authority);
};

/**
 * This function can be used to setup the freeze authority
 * @param connection The Solana RPC connection
 * @param newFreezeAuthority The new freeze authority
 * @param programId The ACCESS program ID
 * @returns ix The instruction to change the freeze authority
 */
export const adminChangeFreezeAuthority = async (
  connection: Connection,
  newFreezeAuthority: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);

  return new adminChangeFreezeAuthorityInstruction({
    newFreezeAuthority: newFreezeAuthority.toBytes(),
  }).getInstruction(programId, centralStateKey, centralState.authority);
};

/**
 * This function can be used to create a V2 bond
 * @param connection The Solana RPC connection
 * @param owner The owner of the bond
 * @param feePayer The fee payer of the transaction
 * @param from The owner of the tokens being bonded
 * @param pool The pool to which the tokens are being bonded
 * @param amount The amount of tokens being bonded
 * @param unlockTimestamp The timestamp at which the tokens can be unlocked if ever. If set to null the tokens are locked forever.
 * @param programId The ACCESS program ID
 * @returns ix The instruction to create the bond V2
 */
export const createBondV2 = async (
  connection: Connection,
  owner: PublicKey,
  feePayer: PublicKey,
  from: PublicKey,
  pool: PublicKey,
  amount: BN,
  unlockTimestamp: BN | null,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const fromAta = getAssociatedTokenAddressSync(
    tokenMint,
    from,
    true,
  );
  const [bondV2Account] = BondV2Account.getKey(programId, owner, pool, unlockTimestamp);
  const centralStateVault = getAssociatedTokenAddressSync(
    tokenMint,
    centralStateKey,
    true,
  );
  const poolVault = getAssociatedTokenAddressSync(
    tokenMint,
    pool,
    true,
  );

  return new createBondV2Instruction({
    amount,
    unlockTimestamp,
  }).getInstruction(
    programId,
    feePayer,
    from,
    fromAta,
    owner,
    bondV2Account,
    centralStateKey,
    centralStateVault,
    pool,
    poolVault,
    tokenMint,
    TOKEN_PROGRAM_ID,
    SystemProgram.programId,
  );
};

/**
 * This function can be used to add tokens to a V2 bond
 * @param connection The Solana RPC connection
 * @param owner The owner of the bond
 * @param from The owner of the tokens being bonded
 * @param pool The pool to which the tokens are being bonded
 * @param amount The amount of tokens being bonded
 * @param unlockTimestamp The timestamp at which the tokens can be unlocked if ever. If set to null the tokens are locked forever.
 * @param programId The ACCESS program ID
 * @returns ix The instruction to add to the bond V2
 */
export const addToBondV2 = async (
  connection: Connection,
  owner: PublicKey,
  from: PublicKey,
  pool: PublicKey,
  amount: BN,
  unlockTimestamp: BN | null,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const fromAta = getAssociatedTokenAddressSync(
    tokenMint,
    from,
    true,
  );
  const [bondV2Account] = BondV2Account.getKey(programId, owner, pool, unlockTimestamp);
  const centralStateVault = getAssociatedTokenAddressSync(
    tokenMint,
    centralStateKey,
    true,
  );
  const poolVault = getAssociatedTokenAddressSync(
    tokenMint,
    pool,
    true,
  );

  return new addToBondV2Instruction({
    amount,
  }).getInstruction(
    programId,
    from,
    fromAta,
    bondV2Account,
    centralStateKey,
    centralStateVault,
    pool,
    poolVault,
    tokenMint,
    TOKEN_PROGRAM_ID,
    SystemProgram.programId,
  );
};

/**
 * This function can be used to claim the rewards of a V2 bond
 * @param connection The Solana RPC connection
 * @param bondAccount The key of the bond account
 * @param programId The ACCESS program ID
 * @returns ix The instruction to claim the bond V2 rewards
 */
export const claimBondV2Rewards = async (
  connection: Connection,
  bondAccount: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const bond = await BondV2Account.retrieve(connection, bondAccount);
  const rewardsDestination = getAssociatedTokenAddressSync(
    tokenMint,
    bond.owner,
    true,
  );

  return new claimBondV2RewardsInstruction().getInstruction(
    programId,
    bond.pool,
    bondAccount,
    bond.owner,
    rewardsDestination,
    centralStateKey,
    tokenMint,
    TOKEN_PROGRAM_ID,
  );
};

/**
 * This function can be used to unlock a V2 bond after the unlock timestamp has passed
 * @param connection The Solana RPC connection
 * @param bondAccount The key of the bond account
 * @param programId The ACCESS program ID
 * @returns ix The instruction to unlock the bond V2
 */
export const unlockBondV2 = async (
  connection: Connection,
  bondAccount: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const bond = await BondV2Account.retrieve(connection, bondAccount);
  const tokenDestination = getAssociatedTokenAddressSync(
    tokenMint,
    bond.owner,
    true,
  );
  const poolVault = getAssociatedTokenAddressSync(
    tokenMint,
    bond.pool,
    true,
  );

  return new unlockBondV2Instruction().getInstruction(
    programId,
    centralStateKey,
    bondAccount,
    bond.owner,
    tokenDestination,
    bond.pool,
    poolVault,
    TOKEN_PROGRAM_ID,
  );
};

/**
 * This function can be used to setup the recipients of the protocol fees
 * @param connection The Solana RPC connection
 * @param recipients The recipients of the protocol fees (address + percentage)
 * @param programId The ACCESS program ID
 * @returns ix The instruction to setup the fee split
 */
export const adminSetupFeeSplit = async (
  connection: Connection,
  recipients: FeeRecipient[],
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);

  return new adminSetupFeeSplitInstruction(
    { recipients },
  ).getInstruction(
    programId,
    centralState.authority,
    centralStateKey,
    SystemProgram.programId,
  )
};

/**
 * This function can be used to distribute the protocol fees
 * @param connection The Solana RPC connection
 * @param programId The ACCESS program ID
 * @returns ix The instruction to distribute the fees
 */
export const distributeFees = async (
  connection: Connection,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }

  const centralStateVault = getAssociatedTokenAddressSync(
    tokenMint,
    centralStateKey,
    true,
  );

  const tokenAccounts = centralState.feeRecipients().map((r) =>
    getAssociatedTokenAddressSync(tokenMint, new PublicKey(r.owner), true))

  return new distributeFeesInstruction().getInstruction(
    programId,
    centralStateKey,
    centralStateVault,
    TOKEN_PROGRAM_ID,
    tokenMint,
    tokenAccounts,
  )
};

/**
 * This function can be used to set the protocol fee
 * @param connection The Solana RPC connection
 * @param protocolFeeBasisPoints The new protocol fee in basis points (i.e. 100 = 1%)
 * @param programId The ACCESS program ID
 * @returns ix The instruction to set the protocol fee
 */
export const adminSetProtocolFee = async (
  connection: Connection,
  protocolFeeBasisPoints: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);

  return new adminSetProtocolFeeInstruction({
    protocolFeeBasisPoints,
  }).getInstruction(
    programId,
    centralState.authority,
    centralStateKey,
  );
};

/**
 * This function can be used to migrate the central state from V1 to V2
 * @param feePayer The fee payer of the transaction
 * @param programId The ACCESS program ID
 * @returns ix The instruction to migrate the central state
 */
export const migrateCentralStateV2 = (
  feePayer: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId); // doesn't matter which, V2 and V1 have the same key
  return new migrateCentralStateV2Instruction().getInstruction(
    programId,
    centralStateKey,
    SystemProgram.programId,
    feePayer,
  );
};

/**
 * This function can be used to freeze or unfreeze the program instructions
 * @param freezeMask The bit mask of the instructions to freeze (0 = freeze, 1 = unfreeze)
 * @param freezeAuthority The authority to freeze the instructions - either the freeze authority (0 mask needed) or the central state authority
 * @param programId The ACCESS program ID
 * @returns ix The instruction to freeze the program instructions
 */
export const adminProgramFreeze = async (
  freezeAuthority: PublicKey,
  freezeMask: BN = new BN.BN(0),
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  return new adminProgramFreezeInstruction({
    ixGate: freezeMask,
  }).getInstruction(
    programId,
    centralStateKey,
    freezeAuthority,
  );
};

/**
 * This function can be used to renounce the admin authority for a specific instruction
 * @param connection The Solana RPC connection
 * @param instruction The instruction to renounce
 * @param programId The ACCESS program ID
 * @returns ix The instruction to renounce the admin authority
 */
export const adminRenounce = async (
  connection: Connection,
  instruction: TaggedInstruction,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);
  return new adminRenounceInstruction({
    ix: instruction.tag,
  }).getInstruction(
    programId,
    centralStateKey,
    centralState.authority,
  );
};