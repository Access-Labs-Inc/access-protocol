// This file is auto-generated. DO NOT EDIT
import BN from "bn.js";
import { Schema, serialize } from "borsh";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}
export class adminMintInstruction {
  tag: number;
  amount: BN;
  static schema: Schema = new Map([
    [
      adminMintInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["amount", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { amount: BN }) {
    this.tag = 17;
    this.amount = obj.amount;
  }
  serialize(): Uint8Array {
    return serialize(adminMintInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    authority: PublicKey,
    mint: PublicKey,
    accessTokenDestination: PublicKey,
    centralState: PublicKey,
    splTokenProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: accessTokenDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class crankInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      crankInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 7;
  }
  serialize(): Uint8Array {
    return serialize(crankInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePool: PublicKey,
    centralState: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class createCentralStateInstruction {
  tag: number;
  dailyInflation: BN;
  authority: Uint8Array;
  static schema: Schema = new Map([
    [
      createCentralStateInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["dailyInflation", "u64"],
          ["authority", [32]],
        ],
      },
    ],
  ]);
  constructor(obj: { dailyInflation: BN; authority: Uint8Array }) {
    this.tag = 0;
    this.dailyInflation = obj.dailyInflation;
    this.authority = obj.authority;
  }
  serialize(): Uint8Array {
    return serialize(createCentralStateInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stateAccount: PublicKey,
    systemProgram: PublicKey,
    feePayer: PublicKey,
    mint: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stateAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: feePayer,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class changeInflationInstruction {
  tag: number;
  dailyInflation: BN;
  static schema: Schema = new Map([
    [
      changeInflationInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["dailyInflation", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { dailyInflation: BN }) {
    this.tag = 10;
    this.dailyInflation = obj.dailyInflation;
  }
  serialize(): Uint8Array {
    return serialize(changeInflationInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    authority: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class closeStakeAccountInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      closeStakeAccountInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 9;
  }
  serialize(): Uint8Array {
    return serialize(closeStakeAccountInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakeAccount: PublicKey,
    owner: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakeAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: owner,
      isSigner: true,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class unlockBondTokensInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      unlockBondTokensInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 13;
  }
  serialize(): Uint8Array {
    return serialize(unlockBondTokensInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    bondAccount: PublicKey,
    bondOwner: PublicKey,
    mint: PublicKey,
    accessTokenDestination: PublicKey,
    centralState: PublicKey,
    splTokenProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: bondAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: bondOwner,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: accessTokenDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class createStakePoolInstruction {
  tag: number;
  owner: Uint8Array;
  destination: Uint8Array;
  minimumStakeAmount: BN;
  static schema: Schema = new Map([
    [
      createStakePoolInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["owner", [32]],
          ["destination", [32]],
          ["minimumStakeAmount", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    owner: Uint8Array;
    destination: Uint8Array;
    minimumStakeAmount: BN;
  }) {
    this.tag = 1;
    this.owner = obj.owner;
    this.destination = obj.destination;
    this.minimumStakeAmount = obj.minimumStakeAmount;
  }
  serialize(): Uint8Array {
    return serialize(createStakePoolInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePoolAccount: PublicKey,
    systemProgram: PublicKey,
    feePayer: PublicKey,
    vault: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakePoolAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: feePayer,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: vault,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class claimBondRewardsInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      claimBondRewardsInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 15;
  }
  serialize(): Uint8Array {
    return serialize(claimBondRewardsInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePool: PublicKey,
    bondAccount: PublicKey,
    bondOwner: PublicKey,
    rewardsDestination: PublicKey,
    centralState: PublicKey,
    mint: PublicKey,
    splTokenProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: bondAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: bondOwner,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: rewardsDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class createBondInstruction {
  tag: number;
  buyer: Uint8Array;
  totalAmountSold: BN;
  totalQuoteAmount: BN;
  quoteMint: Uint8Array;
  sellerTokenAccount: Uint8Array;
  unlockStartDate: BN;
  unlockPeriod: BN;
  unlockAmount: BN;
  lastUnlockTime: BN;
  sellerIndex: BN;
  static schema: Schema = new Map([
    [
      createBondInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["buyer", [32]],
          ["totalAmountSold", "u64"],
          ["totalQuoteAmount", "u64"],
          ["quoteMint", [32]],
          ["sellerTokenAccount", [32]],
          ["unlockStartDate", "u64"],
          ["unlockPeriod", "u64"],
          ["unlockAmount", "u64"],
          ["lastUnlockTime", "u64"],
          ["sellerIndex", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    buyer: Uint8Array;
    totalAmountSold: BN;
    totalQuoteAmount: BN;
    quoteMint: Uint8Array;
    sellerTokenAccount: Uint8Array;
    unlockStartDate: BN;
    unlockPeriod: BN;
    unlockAmount: BN;
    lastUnlockTime: BN;
    sellerIndex: BN;
  }) {
    this.tag = 11;
    this.buyer = obj.buyer;
    this.totalAmountSold = obj.totalAmountSold;
    this.totalQuoteAmount = obj.totalQuoteAmount;
    this.quoteMint = obj.quoteMint;
    this.sellerTokenAccount = obj.sellerTokenAccount;
    this.unlockStartDate = obj.unlockStartDate.fromTwos(64);
    this.unlockPeriod = obj.unlockPeriod.fromTwos(64);
    this.unlockAmount = obj.unlockAmount;
    this.lastUnlockTime = obj.lastUnlockTime.fromTwos(64);
    this.sellerIndex = obj.sellerIndex;
  }
  serialize(): Uint8Array {
    return serialize(createBondInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    seller: PublicKey,
    bondAccount: PublicKey,
    stakePool: PublicKey,
    systemProgram: PublicKey,
    feePayer: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: seller,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: bondAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: feePayer,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class claimPoolRewardsInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      claimPoolRewardsInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 5;
  }
  serialize(): Uint8Array {
    return serialize(claimPoolRewardsInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePool: PublicKey,
    owner: PublicKey,
    rewardsDestination: PublicKey,
    centralState: PublicKey,
    mint: PublicKey,
    splTokenProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: owner,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: rewardsDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class changePoolMinimumInstruction {
  tag: number;
  newMinimum: BN;
  static schema: Schema = new Map([
    [
      changePoolMinimumInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["newMinimum", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { newMinimum: BN }) {
    this.tag = 16;
    this.newMinimum = obj.newMinimum;
  }
  serialize(): Uint8Array {
    return serialize(changePoolMinimumInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePool: PublicKey,
    stakePoolOwner: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: stakePoolOwner,
      isSigner: true,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class stakeInstruction {
  tag: number;
  amount: BN;
  static schema: Schema = new Map([
    [
      stakeInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["amount", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { amount: BN }) {
    this.tag = 3;
    this.amount = obj.amount;
  }
  serialize(): Uint8Array {
    return serialize(stakeInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    centralStateAccount: PublicKey,
    stakeAccount: PublicKey,
    stakePool: PublicKey,
    owner: PublicKey,
    sourceToken: PublicKey,
    splTokenProgram: PublicKey,
    vault: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralStateAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: stakeAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: owner,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: sourceToken,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: vault,
      isSigner: false,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class signBondInstruction {
  tag: number;
  sellerIndex: BN;
  static schema: Schema = new Map([
    [
      signBondInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["sellerIndex", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { sellerIndex: BN }) {
    this.tag = 12;
    this.sellerIndex = obj.sellerIndex;
  }
  serialize(): Uint8Array {
    return serialize(signBondInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    seller: PublicKey,
    bondAccount: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: seller,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: bondAccount,
      isSigner: false,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class claimRewardsInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      claimRewardsInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 6;
  }
  serialize(): Uint8Array {
    return serialize(claimRewardsInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePool: PublicKey,
    stakeAccount: PublicKey,
    owner: PublicKey,
    rewardsDestination: PublicKey,
    centralState: PublicKey,
    mint: PublicKey,
    splTokenProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: stakeAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: owner,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: rewardsDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class closeStakePoolInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      closeStakePoolInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 8;
  }
  serialize(): Uint8Array {
    return serialize(closeStakePoolInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePoolAccount: PublicKey,
    owner: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakePoolAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: owner,
      isSigner: true,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class claimBondInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      claimBondInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 14;
  }
  serialize(): Uint8Array {
    return serialize(claimBondInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    bondAccount: PublicKey,
    buyer: PublicKey,
    quoteTokenSource: PublicKey,
    quoteTokenDestination: PublicKey,
    splTokenProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: bondAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: buyer,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: quoteTokenSource,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: quoteTokenDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class unstakeInstruction {
  tag: number;
  amount: BN;
  static schema: Schema = new Map([
    [
      unstakeInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["amount", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { amount: BN }) {
    this.tag = 4;
    this.amount = obj.amount;
  }
  serialize(): Uint8Array {
    return serialize(unstakeInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    centralStateAccount: PublicKey,
    stakeAccount: PublicKey,
    stakePool: PublicKey,
    owner: PublicKey,
    destinationToken: PublicKey,
    splTokenProgram: PublicKey,
    vault: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralStateAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: stakeAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: owner,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: destinationToken,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: vault,
      isSigner: false,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class createStakeAccountInstruction {
  tag: number;
  nonce: number;
  owner: Uint8Array;
  static schema: Schema = new Map([
    [
      createStakeAccountInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["nonce", "u8"],
          ["owner", [32]],
        ],
      },
    ],
  ]);
  constructor(obj: { nonce: number; owner: Uint8Array }) {
    this.tag = 2;
    this.nonce = obj.nonce;
    this.owner = obj.owner;
  }
  serialize(): Uint8Array {
    return serialize(createStakeAccountInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakeAccount: PublicKey,
    systemProgram: PublicKey,
    stakePool: PublicKey,
    feePayer: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: stakeAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: feePayer,
      isSigner: true,
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
