import {
  activateStakePoolInstruction,
  addToBondV2Instruction,
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
  IndexedInstruction,
  migrateCentralStateV2Instruction,
  stakeInstruction,
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
import BN from "bn.js";
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
    newMinimum: new BN(newMinimum),
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
  programId = ACCESS_PROGRAM_ID,
  rewardsDestination: PublicKey,
  ownerMustSign = true
) => {
  const [centralStateKey] = CentralState.getKey(programId);
  const bond = await BondAccount.retrieve(connection, bondAccount);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralState.retrieve(connection, centralStateKey)).tokenMint;
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

  if (!ownerMustSign) {
    const idx = ix.keys.findIndex((e) => e.pubkey.equals(bond.owner));
    ix.keys[idx].isSigner = false;
  }

  return ix;
};

/**
 * This function can be used by a pool owner to claim his staking rewards
 * @param connection The Solana RPC connection
 * @param stakePoolAccount The key of the stake pool
 * @param rewardsDestination The destination token account for the rewards being claimed
 * @param programId The ACCESS program ID
 * @returns ix The instruction to claim the pool rewards
 */
export const claimPoolRewards = async (
  connection: Connection,
  stakePoolAccount: PublicKey,
  rewardsDestination: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }
  const stakePool = await StakePool.retrieve(connection, stakePoolAccount);

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
 * @param stakeAccount The key of the stake account
 * @param rewardsDestination The destination token account for the rewards being claimed
 * @param programId The ACCESS program ID
 * @returns ix The instruction to claim the rewards
 */
export const claimRewards = async (
  connection: Connection,
  stakeAccount: PublicKey,
  rewardsDestination: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }

  const ix = new claimRewardsInstruction({
    allowZeroRewards: false,
  }).getInstruction(
    programId,
    stake.stakePool,
    stakeAccount,
    stake.owner,
    rewardsDestination,
    centralStateKey,
    tokenMint,
    TOKEN_PROGRAM_ID
  );

  // we don't require the owner to sign this transaction as our use-case is only the pool owner claiming their rewards
  const idx = ix.keys.findIndex((e) => e.pubkey.equals(stake.owner));
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
    dailyInflation: new BN(dailyInflation),
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
    minimumStakeAmount: new BN(minimumStakeAmount),
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
 * @param stakeAccount The key of the stake account
 * @param sourceToken The token account from which the ACS tokens are sent to the stake account
 * @param amount The amount of tokens to stake
 * @param programId The ACCESS program ID
 * @returns
 */
export const stake = async (
  connection: Connection,
  stakeAccount: PublicKey,
  sourceToken: PublicKey,
  amount: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);
  const stakePool = await StakePool.retrieve(connection, stake.stakePool);
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

  return new stakeInstruction({
    amount: new BN(amount),
  }).getInstruction(
    programId,
    centralStateKey,
    stakeAccount,
    stake.stakePool,
    stake.owner,
    sourceToken,
    TOKEN_PROGRAM_ID,
    stakePool.vault,
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
  const [centralStateKey] = CentralState.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralState.retrieve(connection, centralStateKey)).tokenMint;
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
 * @param stakeAccount The key of the stake account
 * @param destinationToken The token account receiving the ACS tokens
 * @param amount The amount of tokens to unlock
 * @param programId The ACCESS program ID
 * @returns
 */
export const unstake = async (
  connection: Connection,
  stakeAccount: PublicKey,
  destinationToken: PublicKey,
  amount: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);
  const stakePool = await StakePool.retrieve(connection, stake.stakePool);
  const [centralStateKey] = CentralStateV2.getKey(programId);

  return new unstakeInstruction({
    amount: new BN(amount),
  }).getInstruction(
    programId,
    centralStateKey,
    stakeAccount,
    stake.stakePool,
    stake.owner,
    destinationToken,
    TOKEN_PROGRAM_ID,
    stakePool.vault,
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
    newMultiplier: new BN(newMultiplier),
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
 * This function can be used to create a V2 bond
 * todo more comments
 */
export const createBondV2 = async (
  connection: Connection,
  owner: PublicKey,
  feePayer: PublicKey,
  from: PublicKey,
  pool: PublicKey,
  amount: number,
  unlockTimestamp: number,
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
  const [bondAccountV2] = BondV2Account.getKey(programId, owner, pool, unlockTimestamp);
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
    bondAccountV2,
    centralStateKey,
    centralStateVault,
    pool,
    poolVault,
    tokenMint,
    TOKEN_PROGRAM_ID,
    SystemProgram.programId,
  );
};

// todo comment
export const addToBondV2 = async (
  connection: Connection,
  owner: PublicKey,
  feePayer: PublicKey,
  from: PublicKey,
  pool: PublicKey,
  amount: number,
  unlockTimestamp: number,
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
  const [bondAccountV2] = BondV2Account.getKey(programId, owner, pool, unlockTimestamp);
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
    unlockTimestamp,
  }).getInstruction(
    programId,
    feePayer,
    from,
    fromAta,
    owner,
    bondAccountV2,
    centralStateKey,
    centralStateVault,
    pool,
    poolVault,
    tokenMint,
    TOKEN_PROGRAM_ID,
    SystemProgram.programId,
  );
};

// todo comment
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
  const bond = await BondAccount.retrieve(connection, bondAccount);
  const rewardsDestination = getAssociatedTokenAddressSync(
    tokenMint,
    bond.owner,
    true,
  );

  return new claimBondV2RewardsInstruction().getInstruction(
    programId,
    bond.stakePool,
    bondAccount,
    bond.owner,
    rewardsDestination,
    centralStateKey,
    tokenMint,
    TOKEN_PROGRAM_ID,
  );
};

// todo comment
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
  const bond = await BondAccount.retrieve(connection, bondAccount);
  const tokenDestination = getAssociatedTokenAddressSync(
    tokenMint,
    bond.owner,
    true,
  );
  const poolVault = getAssociatedTokenAddressSync(
    tokenMint,
    bond.stakePool,
    true,
  );

  return new unlockBondV2Instruction().getInstruction(
    programId,
    centralStateKey,
    bondAccount,
    bond.owner,
    tokenDestination,
    bond.stakePool,
    poolVault,
    TOKEN_PROGRAM_ID,
  );
};

// todo comment
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

// todo comment
export const distributeFees = async (
  connection: Connection,
  feePayer: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);
  const centralStateVault = getAssociatedTokenAddressSync(
    ACCESS_MINT,
    centralStateKey,
    true,
  );

  const tokenAccounts = centralState.recipients.map((r) =>
    getAssociatedTokenAddressSync(ACCESS_MINT, r.owner, true))

  return new distributeFeesInstruction().getInstruction(
    programId,
    feePayer,
    centralStateKey,
    centralStateVault,
    TOKEN_PROGRAM_ID,
    centralState.tokenMint,
    tokenAccounts,
  )
};

// todo comment
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

// todo comment
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

// todo comment
export const adminProgramFreeze = async (
  connection: Connection,
  freezeMask: BN = new BN(0),
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralStateKey);
  return new adminProgramFreezeInstruction({
    ixGate: freezeMask,
  }).getInstruction(
    programId,
    centralStateKey,
    centralState.authority,
  );
};

// todo comment
export const adminRenounce = async (
  connection: Connection,
  instruction: IndexedInstruction,
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