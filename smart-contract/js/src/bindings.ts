import {
  activateStakePoolInstruction,
  changeCentralStateAuthorityInstruction,
  changeInflationInstruction,
  changePoolMinimumInstruction,
  changePoolMultiplierInstruction,
  claimBondRewardsInstruction,
  claimPoolRewardsInstruction,
  claimRewardsInstruction,
  crankInstruction,
  createCentralStateInstruction,
  createStakeAccountInstruction,
  createStakePoolInstruction,
  stakeInstruction,
  unlockBondTokensInstruction,
  unstakeInstruction,
} from "./raw_instructions.js";
import { Connection, PublicKey, SystemProgram } from "@solana/web3.js";
import { ACCESS_MINT, ACCESS_PROGRAM_ID, BondAccount, CentralState, StakeAccount, StakePool } from "./state.js";
import BN from "bn.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

// todo update all comments
/**
 * This function can be used to update the inflation of the central state
 * @param connection The Solana RPC connection
 * @param newInflation The new inflation amount (in micro ACS tokens per day)
 * @param programId The ACCESS program ID
 * @returns
 */
export const adminChangeInflation = async (
  connection: Connection,
  newInflation: BN,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralStateKey);

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
 * @returns
 */
export const changePoolMinimum = async (
  connection: Connection,
  stakePoolKey: PublicKey,
  newMinimum: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stakePool = await StakePool.retrieve(connection, stakePoolKey);
  let [centralStateKey] = CentralState.getKey(programId);

  return new changePoolMinimumInstruction({
    newMinimum: new BN(newMinimum),
  }).getInstruction(programId, stakePoolKey, stakePool.owner, centralStateKey);
};

/**
 * This function can be used activate a created stake pool
 * @param stakePoolKey The key of the stake pool
 * @param programId The ACCESS program ID
 * @returns
 */
export const activateStakePool = (
  stakePoolKey: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralState.getKey(programId);
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
 * @returns
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
 */
export const claimPoolRewards = async (
  connection: Connection,
  stakePoolAccount: PublicKey,
  rewardsDestination: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralStateKey);
  const stakePool = await StakePool.retrieve(connection, stakePoolAccount);

  const ix = new claimPoolRewardsInstruction().getInstruction(
    programId,
    stakePoolAccount,
    stakePool.owner,
    rewardsDestination,
    centralStateKey,
    centralState.tokenMint,
    TOKEN_PROGRAM_ID
  );

  // we don't require the owner to sign this transaction as our use-case is only the pool owner claiming their rewards
  const idx = ix.keys.findIndex((e) => e.pubkey.equals(stakePool.owner));
  ix.keys[idx].isSigner = false;

  return ix;
};

/**
 * This function can be used by a staker to claim his staking rewards
 * @param connection The Solana RPC connection
 * @param stakeAccount The key of the stake account
 * @param rewardsDestination The destination token account for the rewards being claimed
 * @param programId The ACCESS program ID
 * @param allowZeroRewards todo
 * @returns
 */
export const claimRewards = async (
  connection: Connection,
  stakeAccount: PublicKey,
  rewardsDestination: PublicKey,
  programId = ACCESS_PROGRAM_ID,
  allowZeroRewards = false,
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);
  const [centralStateKey] = CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralStateKey);

  const ix = new claimRewardsInstruction({
    allowZeroRewards: allowZeroRewards,
  }).getInstruction(
    programId,
    stake.stakePool,
    stakeAccount,
    stake.owner,
    rewardsDestination,
    centralStateKey,
    centralState.tokenMint,
    TOKEN_PROGRAM_ID
  );

  // we don't require the owner to sign this transaction as our use-case is only the pool owner claiming their rewards
  const idx = ix.keys.findIndex((e) => e.pubkey.equals(stake.owner));
  ix.keys[idx].isSigner = false;

  return ix;
};

/**
 * This function can be used to update the balances of the stake pool
 * @param stakePoolAccount The key fo the stake pool to crank
 * @param programId The ACCESS program ID
 * @returns
 */
export const crank = (
  stakePoolAccount: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralState.getKey(programId);
  return new crankInstruction().getInstruction(
    programId,
    stakePoolAccount,
    centralStateKey
  );
};

/**
 * This function can be used to create the central when deploying the program
 * @param dailyInflation The daily inflation (i.e raw token amounts being emitted per day)
 * @param authority The central state authority (only key that will be able to upgrade the central state)
 * @param mint The ACCESS token mint
 * @param programId The ACCESS program ID
 * @returns
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
  const [centralStateKey] = CentralState.getKey(programId);

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
 * This instruction can be used by content publishers to create their staking pool on which subscription will be based on
 * @param connection The Solana RPC connection
 * @param owner The owner of the stake pool (only key authorized to collect staking rewards)
 * @param minimumStakeAmount The minimum amount of tokens to stake in the pool
 * @param feePayer The fee payer of the tx
 * @param programId The ACCESS program ID
 * @returns
 */
export const createStakePool = async (
  connection: Connection,
  owner: PublicKey,
  minimumStakeAmount: number,
  feePayer: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [stakePool] = StakePool.getKey(programId, owner);
  const [centralStateKey] = CentralState.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralState.retrieve(connection, centralStateKey)).tokenMint;
  }
  const vault = await getAssociatedTokenAddress(
    tokenMint,
    stakePool,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  const createVaultIx = createAssociatedTokenAccountInstruction(
    feePayer,
    vault,
    stakePool,
    tokenMint,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
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
 * This instruction can be used by stakers to deposit ACCESS tokens in their stake account.
 * The staking fee (2%) will be deducted additionaly to the `amount` from the source account.
 * @param connection The Solana RPC connection
 * @param stakeAccount The key of the stake account
 * @param sourceToken The token account from which the ACCESS tokens are sent to the stake account
 * @param amount The raw amount of tokens to stake
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
  const [centralStateKey] = CentralState.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralState.retrieve(connection, centralStateKey)).tokenMint;
  }

  const feesAta = getAssociatedTokenAddressSync(
    tokenMint,
    centralStateKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
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
 * This instruction can be used to request an unstake of ACCESS tokens
 * @param connection The Solana RPC connection
 * @param stakeAccount The key of the stake account
 * @param destinationToken The token account receiving the ACCESS tokens
 * @param amount The amount of tokens to unstake
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
  const [centralStateKey] = CentralState.getKey(programId);

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
 * This function allows a pool owner to adjust the percentage of the pool rewards that go to the pool stakers.
 * @param connection The Solana RPC connection
 * @param stakePoolKey The key of the stake pool
 * @param newMultiplier The new multiplier (in percent [0-100]). This is the percentage of the pools rewards that go to the stakers.
 * @param programId The ACCESS program ID
 * @returns
 */
export const changePoolMultiplier = async (
  connection: Connection,
  stakePoolKey: PublicKey,
  newMultiplier: number,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stakePool = await StakePool.retrieve(connection, stakePoolKey);
  const [centralStateKey] = CentralState.getKey(programId);

  return new changePoolMultiplierInstruction({
    newMultiplier: new BN(newMultiplier),
  }).getInstruction(programId, stakePoolKey, stakePool.owner, centralStateKey);
};

/**
 * This function can be used to change the central state authority
 * @param connection The Solana RPC connection
 * @param newAuthority The new authority of the central state
 * @param programId The ACCESS program ID
 * @returns
 */
export const adminChangeCentralStateAuthority = async (
  connection: Connection,
  newAuthority: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralStateKey);

  return new changeCentralStateAuthorityInstruction({
    newAuthority: newAuthority.toBytes(),
  }).getInstruction(programId, centralStateKey, centralState.authority);
};
