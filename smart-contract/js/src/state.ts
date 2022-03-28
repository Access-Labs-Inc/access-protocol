import { deserialize, Schema } from "borsh";
import BN from "bn.js";
import { Connection, PublicKey } from "@solana/web3.js";
import { u64 } from "./u64";

/**
 * Lenght of the stake pool circular buffer used to store balances and inflation
 */
const STAKE_BUFFER_LEN = 274; // 9 Months

export const MAX_UNSTAKE_REQUEST = 10;

/**
 * Account tags (used for deserialization on-chain)
 */
export enum Tag {
  Uninitialized = 0,
  StakePool = 1,
  InactiveStakePool = 2,
  StakeAccount = 3,
  // Bond accounts are inactive until the buyer transfered the funds
  InactiveBondAccount = 4,
  BondAccount = 5,
  CentralState = 6,
  Deleted = 7,
  FrozenStakePool = 8,
  FrozenStakeAccount = 9,
  FrozenBondAccount = 10,
}

/**
 * Stake pool state
 */
export class RewardsTuple {
  poolReward: BN;
  stakersReward: BN;

  constructor(obj: { poolReward: BN; stakersReward: BN }) {
    this.poolReward = obj.poolReward;
    this.stakersReward = obj.stakersReward;
  }
}

/**
 * Stake pool state
 */
export class StakePool {
  tag: Tag;
  nonce: number;
  currentDayIdx: number;
  _padding: Uint8Array;
  minimumStakeAmount: BN;
  totalStaked: BN;
  lastCrankTime: BN;
  lastClaimedTime: BN;
  stakersPart: BN;
  unstakePeriod: BN;
  owner: PublicKey;
  vault: PublicKey;

  balances: RewardsTuple[];

  static schema: Schema = new Map<any, any>([
    [
      StakePool,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["nonce", "u8"],
          ["currentDayIdx", "u16"],
          ["_padding", [4]],
          ["minimumStakeAmount", "u64"],
          ["totalStaked", "u64"],
          ["lastCrankTime", "u64"],
          ["lastClaimedTime", "u64"],
          ["stakersPart", "u64"],
          ["unstakePeriod", "u64"],
          ["owner", [32]],
          ["vault", [32]],
          ["balances", [RewardsTuple, STAKE_BUFFER_LEN]],
        ],
      },
    ],
    [
      RewardsTuple,
      {
        kind: "struct",
        fields: [
          ["poolReward", "u128"],
          ["stakersReward", "u128"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: number;
    nonce: number;
    currentDayIdx: number;
    _padding: Uint8Array;
    minimumStakeAmount: BN;
    totalStaked: BN;
    totalStakedLastCrank: BN;
    lastCrankTime: BN;
    lastClaimedTime: BN;
    stakersPart: BN;
    unstakePeriod: BN;
    owner: Uint8Array;
    vault: Uint8Array;
    balances: RewardsTuple[];
  }) {
    this.tag = obj.tag as Tag;
    this.nonce = obj.nonce;
    this.currentDayIdx = obj.currentDayIdx;
    this._padding = obj._padding;
    this.minimumStakeAmount = obj.minimumStakeAmount;
    this.totalStaked = obj.totalStaked;
    this.lastCrankTime = obj.lastCrankTime;
    this.lastClaimedTime = obj.lastClaimedTime;
    this.stakersPart = obj.stakersPart;
    this.unstakePeriod = obj.unstakePeriod;
    this.owner = new PublicKey(obj.owner);
    this.vault = new PublicKey(obj.vault);
    this.balances = obj.balances;
  }

  static deserialize(data: Buffer) {
    return deserialize(this.schema, StakePool, data);
  }

  /**
   * This method can be used to retrieve the state of a stake pool
   * @param connection The Solana RPC connection
   * @param key The key of the stake pool
   * @returns
   */
  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("Stake pool not found");
    }
    return this.deserialize(accountInfo.data);
  }

  /**
   * This method can be used to derive the stake pool key
   * @param programId The ACCESS program ID
   * @param owner The owner of the stake pool
   * @returns
   */
  static async getKey(programId: PublicKey, owner: PublicKey) {
    return await PublicKey.findProgramAddress(
      [Buffer.from("stake_pool"), owner.toBuffer()],
      programId
    );
  }
}

/**
 * Unstake request
 */
export class UnstakeRequest {
  amount: BN;
  time: BN;

  constructor(obj: { time: BN; amount: BN }) {
    this.amount = obj.amount;
    this.time = obj.time;
  }
}

/**
 * Stake account state
 */
export class StakeAccount {
  tag: Tag;
  owner: PublicKey;
  stakeAmount: BN;
  stakePool: PublicKey;
  lastClaimedTime: BN;
  poolMinimumAtCreation: BN;
  pendingUnstakeRequests: number;
  unstakeRequests: UnstakeRequest[];

  static schema: Schema = new Map<any, any>([
    [
      UnstakeRequest,
      {
        kind: "struct",
        fields: [
          ["amount", "u64"],
          ["time", "u64"],
        ],
      },
    ],
    [
      StakeAccount,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["owner", [32]],
          ["stakeAmount", "u64"],
          ["stakePool", [32]],
          ["lastClaimedTime", "u64"],
          ["poolMinimumAtCreation", "u64"],
          ["pendingUnstakeRequests", "u8"],
          ["unstakeRequests", [UnstakeRequest, MAX_UNSTAKE_REQUEST]],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: number;
    owner: Uint8Array;
    stakeAmount: BN;
    stakePool: Uint8Array;
    lastClaimedTime: BN;
    poolMinimumAtCreation: BN;
    pendingUnstakeRequests: number;
    unstakeRequests: UnstakeRequest[];
  }) {
    this.tag = obj.tag;
    this.owner = new PublicKey(obj.owner);
    this.stakeAmount = obj.stakeAmount;
    this.stakePool = new PublicKey(obj.stakePool);
    this.lastClaimedTime = obj.lastClaimedTime;
    this.poolMinimumAtCreation = obj.poolMinimumAtCreation;
    this.pendingUnstakeRequests = obj.pendingUnstakeRequests;
    this.unstakeRequests = obj.unstakeRequests;
  }

  static deserialize(data: Buffer) {
    return deserialize(this.schema, StakeAccount, data);
  }

  /**
   * This method can be used to retrieve the state of a stake account
   * @param connection The Solana RPC connection
   * @param key The stake account key
   * @returns
   */
  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("Stake account not found");
    }
    return this.deserialize(accountInfo.data);
  }

  /**
   * This method can be used to derive the stake account key
   * @param programId The ACCESS program ID
   * @param owner The key of the stake account owner
   * @param stakePool The key of the stake pool
   * @returns
   */
  static async getKey(
    programId: PublicKey,
    owner: PublicKey,
    stakePool: PublicKey
  ) {
    return await PublicKey.findProgramAddress(
      [Buffer.from("stake_account"), owner.toBuffer(), stakePool.toBuffer()],
      programId
    );
  }
}

/**
 * The central state
 */
export class CentralState {
  tag: Tag;
  signerNonce: number;
  dailyInflation: BN;
  tokenMint: PublicKey;
  authority: PublicKey;
  totalStaked: BN;

  static schema: Schema = new Map([
    [
      CentralState,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["signerNonce", "u8"],
          ["dailyInflation", "u64"],
          ["tokenMint", [32]],
          ["authority", [32]],
          ["totalStaked", "u64"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: number;
    signerNonce: number;
    dailyInflation: BN;
    tokenMint: Uint8Array;
    authority: Uint8Array;
    totalStaked: BN;
  }) {
    this.tag = obj.tag as Tag;
    this.signerNonce = obj.signerNonce;
    this.dailyInflation = obj.dailyInflation;
    this.tokenMint = new PublicKey(obj.tokenMint);
    this.authority = new PublicKey(obj.authority);
    this.totalStaked = obj.totalStaked;
  }

  static deserialize(data: Buffer) {
    return deserialize(this.schema, CentralState, data);
  }

  /**
   * This method can be used to retrieve the state of the central state
   * @param connection The Solana RPC connection
   * @param key The key of the stake account
   * @returns
   */
  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("Central state not found");
    }
    return this.deserialize(accountInfo.data);
  }

  /**
   * This method can be used to derive the central state key
   * @param programId The ACCESS program ID
   * @returns
   */
  static async getKey(programId: PublicKey) {
    return await PublicKey.findProgramAddress(
      [programId.toBuffer()],
      programId
    );
  }
}

/**
 * The bond account state
 */
export class BondAccount {
  tag: Tag;
  owner: PublicKey;
  totalAmountSold: BN;
  totalStaked: BN;
  totalQuoteAmount: BN;
  quoteMint: PublicKey;
  sellerTokenAccount: PublicKey;
  unlockStartDate: BN;
  unlockPeriod: BN;
  unlockAmount: BN;
  lastUnlockTime: BN;
  totalUnlockedAmount: BN;
  poolMinimumAtCreation: BN;
  stakePool: PublicKey;
  lastClaimedTime: BN;
  sellers: PublicKey[];

  static schema: Schema = new Map([
    [
      BondAccount,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["owner", [32]],
          ["totalAmountSold", "u64"],
          ["totalStaked", "u64"],
          ["totalQuoteAmount", "u64"],
          ["quoteMint", [32]],
          ["sellerTokenAccount", [32]],
          ["unlockStartDate", "u64"],
          ["unlockPeriod", "u64"],
          ["unlockAmount", "u64"],
          ["lastUnlockTime", "u64"],
          ["totalUnlockedAmount", "u64"],
          ["poolMinimumAtCreation", "u64"],
          ["stakePool", [32]],
          ["lastClaimedTime", "u64"],
          ["sellers", [[32]]],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: number;
    owner: Uint8Array;
    totalAmountSold: BN;
    totalStaked: BN;
    totalQuoteAmount: BN;
    quoteMint: Uint8Array;
    sellerTokenAccount: Uint8Array;
    unlockStartDate: BN;
    unlockPeriod: BN;
    unlockAmount: BN;
    lastUnlockTime: BN;
    totalUnlockedAmount: BN;
    poolMinimumAtCreation: BN;
    stakePool: Uint8Array;
    lastClaimedTime: BN;
    sellers: Uint8Array[];
  }) {
    this.tag = obj.tag as Tag;
    this.owner = new PublicKey(obj.owner);
    this.totalAmountSold = obj.totalAmountSold;
    this.totalStaked = obj.totalStaked;
    this.totalQuoteAmount = obj.totalQuoteAmount;
    this.quoteMint = new PublicKey(obj.quoteMint);
    this.sellerTokenAccount = new PublicKey(obj.sellerTokenAccount);
    this.unlockStartDate = obj.unlockStartDate;
    this.unlockPeriod = obj.unlockPeriod;
    this.unlockAmount = obj.unlockAmount;
    this.lastUnlockTime = obj.lastUnlockTime;
    this.totalUnlockedAmount = obj.totalUnlockedAmount;
    this.poolMinimumAtCreation = obj.poolMinimumAtCreation;
    this.stakePool = new PublicKey(obj.stakePool);
    this.lastClaimedTime = obj.lastClaimedTime;
    this.sellers = obj.sellers.map((e) => new PublicKey(e));
  }

  static deserialize(data: Buffer) {
    return deserialize(this.schema, BondAccount, data);
  }

  /**
   * This method can be used to retrieve the state of the bond account
   * @param connection The Solana RPC connection
   * @param key The key of the bond account
   * @returns
   */
  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("Bond account not found");
    }
    return this.deserialize(accountInfo.data);
  }

  /**
   * This method can be used to derive the bond account key
   * @param programId The ACCESS program ID
   * @param owner The owner of the bond
   * @param totalAmountSold The total amount of ACCESS token sold in the bond
   * @returns
   */
  static async getKey(
    programId: PublicKey,
    owner: PublicKey,
    totalAmountSold: number
  ) {
    return await PublicKey.findProgramAddress(
      [
        Buffer.from("bond_account"),
        owner.toBuffer(),
        new u64(totalAmountSold).toBuffer(),
      ],
      programId
    );
  }
}
