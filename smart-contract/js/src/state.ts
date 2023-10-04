import { deserialize, Schema } from "borsh";
import * as BN from 'bn.js';
import { Connection, PublicKey } from "@solana/web3.js";
import { u64 } from "./u64";

/** Default percentage of the staking rewards going to stakers */
export const DEFAULT_STAKER_MULTIPLIER = 50;

/** Length of the circular buffer (stores data for calculating rewards for 274 days) */
export const STAKE_BUFFER_LEN = 274;

/** Maximum count of recipients of the fees */
export const MAX_FEE_RECIPIENTS = 10;

/** Minimum balance of the fee split account allowed for token distribution */
export const MIN_DISTRIBUTE_AMOUNT = 100_000_000;

/** Maximum delay between last fee split distribution and fee split account setup */
export const MAX_FEE_SPLIT_SETUP_DELAY = 5 * 60; // 5 minutes

/** Amount in basis points (i.e 1% = 100) added to each locking operation as a protocol fee */
export const DEFAULT_FEE_BASIS_POINTS = 200;


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
  // V2 tags
  BondV2Account = 11,
  CentralStateV2 = 12,
}

/**
 * Rewards tuple
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
  lastClaimedOffset: BN;
  stakersPart: BN;
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
          ["lastClaimedOffset", "u64"],
          ["stakersPart", "u64"],
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
    lastClaimedOffset: BN;
    stakersPart: BN;
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
    this.lastClaimedOffset = obj.lastClaimedOffset.fromTwos(64);
    this.stakersPart = obj.stakersPart;
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
  static getKey(programId: PublicKey, owner: PublicKey) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("stake_pool"), owner.toBuffer()],
      programId
    );
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
  lastClaimedOffset: BN;
  poolMinimumAtCreation: BN;

  static schema: Schema = new Map<any, any>([
    [
      StakeAccount,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["owner", [32]],
          ["stakeAmount", "u64"],
          ["stakePool", [32]],
          ["lastClaimedOffset", "u64"],
          ["poolMinimumAtCreation", "u64"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: number;
    owner: Uint8Array;
    stakeAmount: BN;
    stakePool: Uint8Array;
    lastClaimedOffset: BN;
    poolMinimumAtCreation: BN;
  }) {
    this.tag = obj.tag;
    this.owner = new PublicKey(obj.owner);
    this.stakeAmount = obj.stakeAmount;
    this.stakePool = new PublicKey(obj.stakePool);
    this.lastClaimedOffset = obj.lastClaimedOffset.fromTwos(64);
    this.poolMinimumAtCreation = obj.poolMinimumAtCreation;
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
  static getKey(
    programId: PublicKey,
    owner: PublicKey,
    stakePool: PublicKey
  ) {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("stake_account"), owner.toBuffer(), stakePool.toBuffer()],
      programId
    );
  }
}

/**
 * The central state V1
 * @deprecated This is the V1 central state, it is deprecated by using the migrateCentralStateV2 instruction
 */
export class CentralState {
  tag: Tag;
  signerNonce: number;
  dailyInflation: BN;
  tokenMint: PublicKey;
  authority: PublicKey;
  creationTime: BN;
  totalStaked: BN;
  totalStakedSnapshot: BN;
  lastSnapshotOffset: BN;

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
          ["creationTime", "u64"],
          ["totalStaked", "u64"],
          ["totalStakedSnapshot", "u64"],
          ["lastSnapshotOffset", "u64"],
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
    creationTime: BN;
    totalStaked: BN;
    totalStakedSnapshot: BN;
    lastSnapshotOffset: BN;
  }) {
    this.tag = obj.tag as Tag;
    this.signerNonce = obj.signerNonce;
    this.dailyInflation = obj.dailyInflation;
    this.tokenMint = new PublicKey(obj.tokenMint);
    this.authority = new PublicKey(obj.authority);
    this.creationTime = obj.creationTime.fromTwos(64);
    this.totalStaked = obj.totalStaked;
    this.totalStakedSnapshot = obj.totalStakedSnapshot.fromTwos(64);
    this.lastSnapshotOffset = obj.lastSnapshotOffset.fromTwos(64);
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
  static getKey(programId: PublicKey) {
    return PublicKey.findProgramAddressSync([programId.toBuffer()], programId);
  }
}

/**
 * The central state V2
 * This can be used only after the migrateCentralStateV2 instruction has been called
 */

export class FeeRecipient {
  owner: PublicKey;
  percentage: BN;

  constructor(obj: {
    owner: PublicKey;
    percentage: BN;
  }) {
    this.owner = new PublicKey(obj.owner);
    this.percentage = obj.percentage;
  }
}

export class CentralStateV2 {
  tag: Tag;
  bumpSeed: number;
  dailyInflation: BN;
  tokenMint: PublicKey;
  authority: PublicKey;
  creationTime: BN;
  totalStaked: BN;
  totalStakedSnapshot: BN;
  lastSnapshotOffset: BN;
  ixGate: BN;
  adminIxGate: BN;
  feeBasisPoints: number;
  lastFeeDistributionTime: BN;
  feeRecipientsCount: number; // this is needed due to Borsh encoding, see https://borsh.io/
  recipients: FeeRecipient[];

  static schema: Schema = new Map<any, any>([
    [
      CentralStateV2,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["bumpSeed", "u8"],
          ["dailyInflation", "u64"],
          ["tokenMint", [32]],
          ["authority", [32]],
          ["creationTime", "u64"],
          ["totalStaked", "u64"],
          ["totalStakedSnapshot", "u64"],
          ["lastSnapshotOffset", "u64"],
          ["ixGate", "u128"],
          ["adminIxGate", "u128"],
          ["feeBasisPoints", "u16"],
          ["lastFeeDistributionTime", "u64"],
          ["feeRecipientsCount", "u32"],
          ["recipients", [FeeRecipient, MAX_FEE_RECIPIENTS]],
        ],
      },
    ],
    [
      FeeRecipient,
      {
        kind: "struct",
        fields: [
          ["owner", [32]],
          ["percentage", "u64"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: number;
    bumpSeed: number;
    dailyInflation: BN;
    tokenMint: Uint8Array;
    authority: Uint8Array;
    creationTime: BN;
    totalStaked: BN;
    totalStakedSnapshot: BN;
    lastSnapshotOffset: BN;
    ixGate: BN;
    adminIxGate: BN;
    feeBasisPoints: number;
    lastFeeDistributionTime: BN;
    feeRecipientsCount: number;
    recipients: FeeRecipient[];
  }) {
    this.tag = obj.tag as Tag;
    this.bumpSeed = obj.bumpSeed;
    this.dailyInflation = obj.dailyInflation;
    this.tokenMint = new PublicKey(obj.tokenMint);
    this.authority = new PublicKey(obj.authority);
    this.creationTime = obj.creationTime.fromTwos(64);
    this.totalStaked = obj.totalStaked;
    this.totalStakedSnapshot = obj.totalStakedSnapshot;
    this.lastSnapshotOffset = obj.lastSnapshotOffset;
    this.ixGate = obj.ixGate;
    this.adminIxGate = obj.adminIxGate;
    this.feeBasisPoints = obj.feeBasisPoints;
    this.lastFeeDistributionTime = obj.lastFeeDistributionTime.fromTwos(64);
    this.feeRecipientsCount = obj.feeRecipientsCount;
    this.recipients = obj.recipients;
  }

  static deserialize(data: Buffer) {
    return deserialize(this.schema, CentralStateV2, data);
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
      throw new Error("Central state V2 not found");
    }
    return this.deserialize(accountInfo.data);
  }

  /**
   * This method can be used to derive the central state key
   * @param programId The ACCESS program ID
   * @returns
   */
  static getKey(programId: PublicKey) {
    return PublicKey.findProgramAddressSync([programId.toBuffer()], programId);
  }

  calculateFee(amount: BN) {
    return amount
      .muln(this.feeBasisPoints)
      .addn(9_999)
      .divn(10_000);
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
  lastClaimedOffset: BN;
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
          ["lastClaimedOffset", "u64"],
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
    lastClaimedOffset: BN;
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
    this.lastClaimedOffset = obj.lastClaimedOffset;
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
  static getKey(
    programId: PublicKey,
    owner: PublicKey,
    totalAmountSold: number
  ) {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("bond_account"),
        owner.toBuffer(),
        new u64(totalAmountSold).toBuffer(),
      ],
      programId
    );
  }
}

/**
 * The bond V2 state
 */

export class BondV2Account {
  tag: Tag;
  owner: PublicKey;
  amount: BN;
  pool: PublicKey;
  lastClaimedOffset: BN;
  poolMinimumAtCreation: BN;
  unlockTimestamp: null | BN; // todo check if this is a possible representation of Option<i64>

  static schema: Schema = new Map<any, any>([
    [
      BondV2Account,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["owner", [32]],
          ["amount", "u64"],
          ["pool", [32]],
          ["lastClaimedOffset", "u64"],
          ["poolMinimumAtCreation", "u64"],
          ["unlockTimestamp", "Option<i64>"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    tag: number;
    owner: Uint8Array;
    amount: BN;
    pool: Uint8Array;
    lastClaimedOffset: BN;
    poolMinimumAtCreation: BN;
    unlockTimestamp: null | BN;
  }) {
    this.tag = obj.tag;
    this.owner = new PublicKey(obj.owner);
    this.amount = obj.amount;
    this.pool = new PublicKey(obj.pool);
    this.lastClaimedOffset = obj.lastClaimedOffset.fromTwos(64);
    this.poolMinimumAtCreation = obj.poolMinimumAtCreation;
    this.unlockTimestamp = obj.unlockTimestamp;
  }

  static deserialize(data: Buffer) {
    return deserialize(this.schema, BondV2Account, data);
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
   * @param unlockTimestamp todo
   * @returns
   */
  static getKey(
    programId: PublicKey,
    owner: PublicKey,
    stakePool: PublicKey,
    unlockTimestamp: null | BN,
  ) {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("bond_v2_account"),
        owner.toBuffer(),
        stakePool.toBuffer(),
        (unlockTimestamp ? unlockTimestamp : new BN.BN(0)).toBuffer()], // todo check if new BN.BN(0) is right for the forever bonds
      programId
    );
  }
}


/// mainnet ACCESS token mint and program id
export const ACCESS_MINT = new PublicKey("5MAYDfq5yxtudAhtfyuMBuHZjgAbaS9tbEyEQYAhDS5y");
export const ACCESS_PROGRAM_ID = new PublicKey("6HW8dXjtiTGkD4jzXs7igdFmZExPpmwUrRN5195xGup");