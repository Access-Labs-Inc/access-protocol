import {
  changeInflationInstruction,
  changePoolMinimumInstruction,
  claimBondRewardsInstruction,
  claimBondInstruction,
  claimPoolRewardsInstruction,
  claimRewardsInstruction,
  closeStakeAccountInstruction,
  closeStakePoolInstruction,
  crankInstruction,
  createBondInstruction,
  createCentralStateInstruction,
  createStakeAccountInstruction,
  createStakePoolInstruction,
  signBondInstruction,
  stakeInstruction,
  unlockBondTokensInstruction,
  unstakeInstruction,
  adminMintInstruction,
  activateStakePoolInstruction,
  adminFreezeInstruction,
  changePoolMultiplierInstruction,
  changeCentralStateAuthorityInstruction,
  editMetadataInstruction,
} from "./raw_instructions";
import {
  Connection,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { CentralState, StakePool, BondAccount, StakeAccount } from "./state";
import BN from "bn.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction, getImmutableOwner,
} from "@solana/spl-token";
import { findMetadataPda, TokenMetadataProgram } from "@metaplex-foundation/js";
import {u64} from "./u64";
import {getBondAccounts} from "./secondary_bindings";

/**
 * This function can be used to update the inflation schedule of the central state
 * @param connection The Solana RPC connection
 * @param newInflation The new inflation amount (in raw token amounts per day)
 * @param programId The ACCESS program ID
 * @returns
 */
export const changeInflation = async (
  connection: Connection,
  newInflation: BN,
  programId: PublicKey
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);

  const ix = new changeInflationInstruction({
    dailyInflation: newInflation,
  }).getInstruction(programId, centralKey, centralState.authority);

  return ix;
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
  programId: PublicKey
) => {
  const stakePool = await StakePool.retrieve(connection, stakePoolKey);

  const ix = new changePoolMinimumInstruction({
    newMinimum: new BN(newMinimum),
  }).getInstruction(programId, stakePoolKey, stakePool.owner);

  return ix;
};

/**
 * This function can be used activate a created stake pool with the signing authority
 * @param connection The Solana RPC connection
 * @param stakePoolKey The key of the stake pool
 * @param programId The ACCESS program ID
 * @returns
 */
export const activateStakePool = async (
  connection: Connection,
  stakePoolKey: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);

  const ix = new activateStakePoolInstruction().getInstruction(
    programId,
    centralState.authority,
    stakePoolKey,
    centralKey
  );

  return ix;
};

/**
 * This function can be used by a bond owner to claim his staking rewards
 * @param connection The Solana RPC connection
 * @param bondAccount The key of the bond account
 * @param rewardsDestination The destination token account for the rewards being claimed
 * @param programId The ACCESS program ID
 * @returns
 */
export const claimBondRewards = async (
  connection: Connection,
  bondAccount: PublicKey,
  rewardsDestination: PublicKey,
  programId: PublicKey,
  ownerMustSign = true
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);

  const bond = await BondAccount.retrieve(connection, bondAccount);

  const ix = new claimBondRewardsInstruction().getInstruction(
    programId,
    bond.stakePool,
    bondAccount,
    bond.owner,
    rewardsDestination,
    centralKey,
    centralState.tokenMint,
    TOKEN_PROGRAM_ID
  );

  if (!ownerMustSign) {
    const idx = ix.keys.findIndex((e) => e.pubkey.equals(bond.owner));
    ix.keys[idx].isSigner = false;
  }

  return ix;
};

/**
 * This function can be used by a bond buyer to claim his bond
 * @param connection The Solana RPC connection
 * @param bondAccount The key of the bond account
 * @param buyer The key of the bond buyer
 * @param quoteTokenSource The token account used to purchase the bond
 * @param programId The ACCESS program ID
 * @returns
 */
export const claimBond = async (
  connection: Connection,
  bondAccount: PublicKey,
  buyer: PublicKey,
  quoteTokenSource: PublicKey,
  programId: PublicKey
) => {
  const bond = await BondAccount.retrieve(connection, bondAccount);
  const stakePool = await StakePool.retrieve(connection, bond.stakePool);
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);

  const ix = new claimBondInstruction().getInstruction(
    programId,
    bondAccount,
    buyer,
    quoteTokenSource,
    bond.sellerTokenAccount,
    bond.stakePool,
    centralState.tokenMint,
    stakePool.vault,
    centralKey,
    TOKEN_PROGRAM_ID
  );

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
  programId: PublicKey,
  ownerMustSign = true
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);
  const stakePool = await StakePool.retrieve(connection, stakePoolAccount);

  const ix = new claimPoolRewardsInstruction().getInstruction(
    programId,
    stakePoolAccount,
    stakePool.owner,
    rewardsDestination,
    centralKey,
    centralState.tokenMint,
    TOKEN_PROGRAM_ID
  );

  if (!ownerMustSign) {
    const idx = ix.keys.findIndex((e) => e.pubkey.equals(stakePool.owner));
    ix.keys[idx].isSigner = false;
  }

  return ix;
};

/**
 * This function can be used by a staker to claim his staking rewards
 * @param connection The Solana RPC connection
 * @param stakeAccount The key of the stake account
 * @param rewardsDestination The destination token account for the rewards being claimed
 * @param programId The ACCESS program ID
 * @returns
 */
export const claimRewards = async (
  connection: Connection,
  stakeAccount: PublicKey,
  rewardsDestination: PublicKey,
  programId: PublicKey,
  allowZeroRewards: boolean,
  ownerMustSign = true
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);

  const ix = new claimRewardsInstruction({
    allowZeroRewards: Number(allowZeroRewards),
  }).getInstruction(
    programId,
    stake.stakePool,
    stakeAccount,
    stake.owner,
    rewardsDestination,
    centralKey,
    centralState.tokenMint,
    TOKEN_PROGRAM_ID
  );

  if (!ownerMustSign) {
    const idx = ix.keys.findIndex((e) => e.pubkey.equals(stake.owner));
    ix.keys[idx].isSigner = false;
  }

  return ix;
};

/**
 * This function can be used by a staker to close his stake account and collect its rent
 * @param connection The Solana RPC connection
 * @param stakeAccount The key of the stake account
 * @param programId The ACCESS program ID
 * @returns
 */
export const closeStakeAccount = async (
  connection: Connection,
  stakeAccount: PublicKey,
  programId: PublicKey
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);

  const ix = new closeStakeAccountInstruction().getInstruction(
    programId,
    stakeAccount,
    stake.owner
  );

  return ix;
};

/**
 * This function can be used by a stake pool owner to close the pool and collect its rent
 * @param connection The Solana RPC connection
 * @param stakePoolAccount The key of the stake pool
 * @param programId The ACCESS program ID
 * @returns
 */
export const closeStakePool = async (
  connection: Connection,
  stakePoolAccount: PublicKey,
  programId: PublicKey
) => {
  const stakePool = await StakePool.retrieve(connection, stakePoolAccount);

  const ix = new closeStakePoolInstruction().getInstruction(
    programId,
    stakePoolAccount,
    stakePool.vault,
    stakePool.owner
  );

  return ix;
};

/**
 * This function can be used to update the balances of the stake pool
 * @param stakePoolAccount The key fo the stake pool to crank
 * @param programId The ACCESS program ID
 * @returns
 */
export const crank = async (
  stakePoolAccount: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const ix = new crankInstruction().getInstruction(
    programId,
    stakePoolAccount,
    centralKey
  );

  return ix;
};

/**
 * This function can be used to issue ACCESS locked tokens (bonds)
 * @param seller The initial bond seller
 * @param buyer The bond buyer
 * @param totalAmountSold The total amount of ACCESS tokens being sold
 * @param totalQuoteAmount The total amount of quote tokens used to buy the bond
 * @param quoteMint The mint of the token used to buy the bond
 * @param sellerTokenAccount The seller token account (used to collect proceeds of the sale)
 * @param unlockStartDate The unix timestamp (in s) at which the tokens start unlock
 * @param unlockPeriod The time interval at which the tokens unlock
 * @param unlockAmount The amount that unlocks at each period
 * @param lastUnlockTime The unix timestamp at which the unlock stops
 * @param stakePool The stake pool key
 * @param sellerIndex The seller index in the array of authorized sellers
 * @param programId The ACCESS program ID
 * @returns
 */
export const createBond = async (
  seller: PublicKey,
  buyer: PublicKey,
  totalAmountSold: number,
  totalQuoteAmount: number,
  quoteMint: PublicKey,
  sellerTokenAccount: PublicKey,
  unlockStartDate: number,
  unlockPeriod: number,
  unlockAmount: number,
  stakePool: PublicKey,
  sellerIndex: number,
  programId: PublicKey
) => {
  const [bondAccount] = await BondAccount.getKey(
    programId,
    buyer,
    totalAmountSold
  );

  const ix = new createBondInstruction({
    buyer: buyer.toBuffer(),
    totalAmountSold: new BN(totalAmountSold),
    totalQuoteAmount: new BN(totalQuoteAmount),
    quoteMint: quoteMint.toBuffer(),
    sellerTokenAccount: sellerTokenAccount.toBuffer(),
    unlockStartDate: new BN(unlockStartDate),
    unlockPeriod: new BN(unlockPeriod),
    unlockAmount: new BN(unlockAmount),
    sellerIndex: new BN(sellerIndex),
  }).getInstruction(
    programId,
    seller,
    bondAccount,
    stakePool,
    SystemProgram.programId,
    seller
  );

  return ix;
};

/**
 * This function can be used to create the central when deploying the program
 * @param dailyInflation The daily inflation (i.e raw token amounts being emitted per day)
 * @param authority The central state authority (only key that will be able to upgrade the central state)
 * @param feePayer The fee payer of the tx
 * @param mint The ACCESS token mint
 * @param name The name of the ACCESS token
 * @param symbol The symbol of the ACCESS token
 * @param uri The URI of the ACCESS metadata
 * @param programId The ACCESS program ID
 * @returns
 */
export const createCentralState = async (
  dailyInflation: number,
  authority: PublicKey,
  feePayer: PublicKey,
  mint: PublicKey,
  programId: PublicKey,
) => {
  const [centralKey] = await CentralState.getKey(programId);

  const ix = new createCentralStateInstruction({
    dailyInflation: new BN(dailyInflation),
    authority: authority.toBuffer(),
  }).getInstruction(
    programId,
    centralKey,
    SystemProgram.programId,
    feePayer,
    mint,
  );

  return ix;
};

/**
 * This function can be used to create a stake account in stake pool
 * @param stakePool The key of the stake pool
 * @param owner The owner of the staking account being created (i.e the staker)
 * @param feePayer The fee payer of the transaction
 * @param programId The ACCESS program ID
 * @returns
 */
export const createStakeAccount = async (
  stakePool: PublicKey,
  owner: PublicKey,
  feePayer: PublicKey,
  programId: PublicKey
) => {
  const [stakeAccount, nonce] = await StakeAccount.getKey(
    programId,
    owner,
    stakePool
  );

  const ix = new createStakeAccountInstruction({
    nonce,
    owner: owner.toBuffer(),
  }).getInstruction(
    programId,
    stakeAccount,
    SystemProgram.programId,
    stakePool,
    feePayer
  );

  return ix;
};

/**
 * This instruction can be used by content publishers to create their staking pool on which subscription will be based on
 * @param connection The Solana RPC connection
 * @param name The name of the stake pool
 * @param owner The owner of the stake pool (only key authorized to collect staking rewards)
 * @param destination The destination of the stake pool rewards
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
  programId: PublicKey
) => {
  const [stakePool] = await StakePool.getKey(programId, owner);
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);
  const vault = await getAssociatedTokenAddress(
    centralState.tokenMint,
    stakePool,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  const createVaultIx = createAssociatedTokenAccountInstruction(
    feePayer,
    vault,
    stakePool,
    centralState.tokenMint,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  const ix = new createStakePoolInstruction({
    owner: owner.toBuffer(),
    minimumStakeAmount: new BN(minimumStakeAmount),
  }).getInstruction(
    programId,
    stakePool,
    SystemProgram.programId,
    feePayer,
    vault
  );

  return [createVaultIx, ix];
};

/**
 * This instruction can be used by authorized sellers to approve the sell of a bond
 * @param sellerIndex The index of the seller in the array of authorized sellers
 * @param seller The seller key
 * @param bondAccount The bond account key
 * @param programId The ACCESS program ID
 * @returns
 */
export const signBond = async (
  sellerIndex: number,
  seller: PublicKey,
  bondAccount: PublicKey,
  programId: PublicKey
) => {
  const ix = new signBondInstruction({
    sellerIndex: new BN(sellerIndex),
  }).getInstruction(programId, seller, bondAccount);

  return ix;
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
  programId: PublicKey
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);
  const stakePool = await StakePool.retrieve(connection, stake.stakePool);
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);
  const bondAccounts = await getBondAccounts(connection, stake.owner, programId);

  const bondAccountKey = bondAccounts.find(
    bond =>
      BondAccount.deserialize(bond.account.data).stakePool.toBase58() ===
      stake.stakePool.toBase58(),
  )?.pubkey;

  const feesAta = await getAssociatedTokenAddress(
    centralState.tokenMint,
    centralState.authority,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  const ix = new stakeInstruction({
    amount: new BN(amount),
  }).getInstruction(
    programId,
    centralKey,
    stakeAccount,
    stake.stakePool,
    stake.owner,
    sourceToken,
    TOKEN_PROGRAM_ID,
    stakePool.vault,
    feesAta,
    bondAccountKey,
  );

  return ix;
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
  programId: PublicKey
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);
  const bond = await BondAccount.retrieve(connection, bondAccount);
  const stakePool = await StakePool.retrieve(connection, bond.stakePool);

  const ix = new unlockBondTokensInstruction().getInstruction(
    programId,
    bondAccount,
    bond.owner,
    centralState.tokenMint,
    destinationToken,
    centralKey,
    bond.stakePool,
    stakePool.vault,
    TOKEN_PROGRAM_ID
  );

  return ix;
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
  programId: PublicKey
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);
  const stakePool = await StakePool.retrieve(connection, stake.stakePool);
  const [centralKey] = await CentralState.getKey(programId);
  const bondAccounts = await getBondAccounts(connection, stake.owner, programId);
  const bondAccountKey = bondAccounts.find(
    bond =>
      BondAccount.deserialize(bond.account.data).stakePool.toBase58() ===
      stake.stakePool.toBase58(),
  )?.pubkey;

  const ix = new unstakeInstruction({
    amount: new BN(amount)
  }).getInstruction(
    programId,
    centralKey,
    stakeAccount,
    stake.stakePool,
    stake.owner,
    destinationToken,
    TOKEN_PROGRAM_ID,
    stakePool.vault,
    bondAccountKey
  );

  return ix;
};

/**
 * This instruction can be used to mint ACCESS tokens. It requires the central state authority to sign.
 * @param connection The Solana RPC connection
 * @param amount The amount of tokens to mint
 * @param destinationToken The token account receiving the ACCESS tokens
 * @param programId The ACCESS program ID
 * @returns
 */
export const adminMint = async (
  connection: Connection,
  amount: number,
  destinationToken: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);

  const ix = new adminMintInstruction({
    amount: new BN(amount),
  }).getInstruction(
    programId,
    centralState.authority,
    centralState.tokenMint,
    destinationToken,
    centralKey,
    TOKEN_PROGRAM_ID
  );

  return ix;
};

/**
 * This instruction can be used by the central state authority to freeze or unfreeze an account
 * @param connection The Solana RPC connection
 * @param accountToFreeze The account to freeze
 * @param programId The ACCESS program ID
 * @returns
 */
export const adminFreeze = async (
  connection: Connection,
  accountToFreeze: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);

  const ix = new adminFreezeInstruction().getInstruction(
    programId,
    centralState.authority,
    accountToFreeze,
    centralKey
  );

  return ix;
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
  programId: PublicKey
) => {
  const stakePool = await StakePool.retrieve(connection, stakePoolKey);

  const ix = new changePoolMultiplierInstruction({
    newMultiplier: new BN(newMultiplier),
  }).getInstruction(programId, stakePoolKey, stakePool.owner);

  return ix;
};

/**
 * This function can be used to change the central state authority
 * @param connection The Solana RPC connection
 * @param newAuthority The new authority of the central state
 * @param programId The ACCESS program ID
 * @returns
 */
export const changeCentralStateAuthority = async (
  connection: Connection,
  newAuthority: PublicKey,
  programId: PublicKey
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);

  const ix = new changeCentralStateAuthorityInstruction({
    newAuthority: newAuthority.toBytes(),
  }).getInstruction(programId, centralKey, centralState.authority);

  return ix;
};

/**
 * This function can be used to edit the token metadata
 * @param connection The Solana RPC connection
 * @param name The new metadata name
 * @param symbol The new metadata symbol
 * @param uri The new metadata URI
 * @param programId The ACCESS program ID
 * @param tokenMetataProgramId The token metadata program ID (optional)
 * @returns
 */
export const editMetadata = async (
  connection: Connection,
  name: string,
  symbol: string,
  uri: string,
  programId: PublicKey,
  tokenMetataProgramId = TokenMetadataProgram.publicKey
) => {
  const [centralKey] = await CentralState.getKey(programId);
  const centralState = await CentralState.retrieve(connection, centralKey);
  const metadata = findMetadataPda(
    centralState.tokenMint,
    tokenMetataProgramId
  );

  const ix = new editMetadataInstruction({ name, symbol, uri }).getInstruction(
    programId,
    centralKey,
    centralState.authority,
    metadata,
    tokenMetataProgramId
  );

  return ix;
};
