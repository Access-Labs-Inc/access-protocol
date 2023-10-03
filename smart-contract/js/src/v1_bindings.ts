/**
 * @file This file contains the bindings for the v1 smart contract that were made obsolete in the v2 smart contract.
 * this file is only used for testing the v1 to v2 migrations.
 */

import {
  adminFreezeInstruction,
  adminMintInstruction,
  claimBondInstruction,
  closeStakeAccountInstruction,
  closeStakePoolInstruction,
  createBondInstruction,
  signBondInstruction,
} from "./raw_instructions";
import { Connection, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  ACCESS_MINT,
  ACCESS_PROGRAM_ID,
  BondAccount,
  CentralStateV2,
  StakeAccount,
  StakePool
} from "./state";
import * as BN from 'bn.js';
import { TOKEN_PROGRAM_ID, } from "@solana/spl-token";


/**
 * This function can be used by a bond buyer to claim his bond
 * @param connection The Solana RPC connection
 * @param bondAccount The key of the bond account
 * @param buyer The key of the bond buyer
 * @param quoteTokenSource The token account used to purchase the bond
 * @param programId The ACCESS program ID
 * @returns ix The claim bond instruction
 * @deprecated This function is deprecated in the v2 smart contract
 */
export const claimBond = async (
  connection: Connection,
  bondAccount: PublicKey,
  buyer: PublicKey,
  quoteTokenSource: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const bond = await BondAccount.retrieve(connection, bondAccount);
  const stakePool = await StakePool.retrieve(connection, bond.stakePool);
  const [centralStateKey] = CentralStateV2.getKey(programId);
  let tokenMint = ACCESS_MINT;
  if (programId !== ACCESS_PROGRAM_ID) {
    tokenMint = (await CentralStateV2.retrieve(connection, centralStateKey)).tokenMint;
  }

  return new claimBondInstruction().getInstruction(
    programId,
    bondAccount,
    buyer,
    quoteTokenSource,
    bond.sellerTokenAccount,
    bond.stakePool,
    tokenMint,
    stakePool.vault,
    centralStateKey,
    TOKEN_PROGRAM_ID
  );
};


/**
 * This function can be used by a staker to close his stake account and collect its rent
 * @param connection The Solana RPC connection
 * @param stakeAccount The key of the stake account
 * @param programId The ACCESS program ID
 * @returns ix The close stake account instruction
 * @deprecated This function is deprecated in the v2 smart contract
 */
export const closeStakeAccount = async (
  connection: Connection,
  stakeAccount: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stake = await StakeAccount.retrieve(connection, stakeAccount);
  const [centralStateKey] = CentralStateV2.getKey(programId);

  return new closeStakeAccountInstruction().getInstruction(
    programId,
    stakeAccount,
    stake.owner,
    centralStateKey,
  );
};

/**
 * This function can be used by a stake pool owner to close the pool and collect its rent
 * @param connection The Solana RPC connection
 * @param stakePoolAccount The key of the stake pool
 * @param programId The ACCESS program ID
 * @returns ix The close stake pool instruction
 * @deprecated This function is deprecated in the v2 smart contract
 */
export const closeStakePool = async (
  connection: Connection,
  stakePoolAccount: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const stakePool = await StakePool.retrieve(connection, stakePoolAccount);
  const [centralStateKey] = CentralStateV2.getKey(programId);

  return new closeStakePoolInstruction().getInstruction(
    programId,
    stakePoolAccount,
    stakePool.vault,
    stakePool.owner,
    centralStateKey,
  );
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
 * @param stakePool The stake pool key
 * @param sellerIndex The seller index in the array of authorized sellers
 * @param programId The ACCESS program ID
 * @returns ix The create bond instruction
 * @deprecated This function is deprecated in the v2 smart contract
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
  programId = ACCESS_PROGRAM_ID,
) => {
  const [bondAccount] = BondAccount.getKey(
    programId,
    buyer,
    totalAmountSold
  );
  const [centralStateKey] = CentralStateV2.getKey(programId);

  return new createBondInstruction({
    buyer: buyer.toBuffer(),
    totalAmountSold: new BN.BN(totalAmountSold),
    totalQuoteAmount: new BN.BN(totalQuoteAmount),
    quoteMint: quoteMint.toBuffer(),
    sellerTokenAccount: sellerTokenAccount.toBuffer(),
    unlockStartDate: new BN.BN(unlockStartDate),
    unlockPeriod: new BN.BN(unlockPeriod),
    unlockAmount: new BN.BN(unlockAmount),
    sellerIndex: new BN.BN(sellerIndex),
  }).getInstruction(
    programId,
    seller,
    bondAccount,
    stakePool,
    SystemProgram.programId,
    seller,
    centralStateKey,
  );
};

/**
 * This instruction can be used by authorized sellers to approve the sell of a bond
 * @param sellerIndex The index of the seller in the array of authorized sellers
 * @param seller The seller key
 * @param bondAccount The bond account key
 * @param programId The ACCESS program ID
 * @returns ix The sign bond instruction
 * @deprecated This function is deprecated in the v2 smart contract
 */
export const signBond = async (
  sellerIndex: number,
  seller: PublicKey,
  bondAccount: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralStateKey] = CentralStateV2.getKey(programId);
  return new signBondInstruction({
    sellerIndex: new BN.BN(sellerIndex),
  }).getInstruction(programId, seller, bondAccount, centralStateKey);
};


/**
 * This instruction can be used to mint ACCESS tokens. It requires the central state authority to sign.
 * @param connection The Solana RPC connection
 * @param amount The amount of tokens to mint
 * @param destinationToken The token account receiving the ACCESS tokens
 * @param programId The ACCESS program ID
 * @returns ix The mint instruction
 * @deprecated This function is deprecated in the v2 smart contract
 */
export const adminMint = async (
  connection: Connection,
  amount: number,
  destinationToken: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralKey);

  return new adminMintInstruction({
    amount: new BN.BN(amount),
  }).getInstruction(
    programId,
    centralState.authority,
    centralState.tokenMint,
    destinationToken,
    centralKey,
    TOKEN_PROGRAM_ID
  );
};

/**
 * This instruction can be used by the central state authority to freeze or unfreeze an account
 * @param connection The Solana RPC connection
 * @param accountToFreeze The account to freeze
 * @param programId The ACCESS program ID
 * @returns
 * @deprecated This function is deprecated in the v2 smart contract
 */
export const adminFreeze = async (
  connection: Connection,
  accountToFreeze: PublicKey,
  programId = ACCESS_PROGRAM_ID,
) => {
  const [centralKey] = CentralStateV2.getKey(programId);
  const centralState = await CentralStateV2.retrieve(connection, centralKey);

  return new adminFreezeInstruction().getInstruction(
    programId,
    centralState.authority,
    accountToFreeze,
    centralKey
  );
};

