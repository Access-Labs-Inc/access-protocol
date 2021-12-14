// This file is auto-generated. DO NOT EDIT
import BN from "bn.js";
import { Schema, serialize } from "borsh";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}
export class closeStakePoolInstruction {
  tag: number;
  nonce: number;
  name: string;
  destination: number;
  static schema: Schema = new Map([
    [
      closeStakePoolInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["nonce", "u8"],
          ["name", "string"],
          ["destination", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: { nonce: number; name: string; destination: number }) {
    this.tag = 8;
    this.nonce = obj.nonce;
    this.name = obj.name;
    this.destination = obj.destination;
  }
  serialize(): Uint8Array {
    return serialize(closeStakePoolInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePoolAccount: PublicKey,
    systemProgram: PublicKey,
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
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
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
  constructor(obj: {}) {
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
    centralVault: PublicKey,
    sourceRewards: PublicKey,
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
      isWritable: true,
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
      isWritable: false,
    });
    keys.push({
      pubkey: centralVault,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: sourceRewards,
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
  constructor(obj: {}) {
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
    centralVault: PublicKey,
    sourceRewards: PublicKey,
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
      isWritable: true,
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
      isWritable: false,
    });
    keys.push({
      pubkey: centralVault,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: sourceRewards,
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
      isWritable: true,
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
export class createCentralStateInstruction {
  tag: number;
  dailyInflation: BN;
  authority: number;
  static schema: Schema = new Map([
    [
      createCentralStateInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["dailyInflation", "u64"],
          ["authority", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: { dailyInflation: BN; authority: number }) {
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
    rentSysvarAccount: PublicKey,
    centralVault: PublicKey,
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
      pubkey: rentSysvarAccount,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: centralVault,
      isSigner: false,
      isWritable: false,
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
export class closeStakeAccountInstruction {
  tag: number;
  nonce: number;
  owner: number;
  stakePool: number;
  static schema: Schema = new Map([
    [
      closeStakeAccountInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["nonce", "u8"],
          ["owner", "u8"],
          ["stakePool", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: { nonce: number; owner: number; stakePool: number }) {
    this.tag = 9;
    this.nonce = obj.nonce;
    this.owner = obj.owner;
    this.stakePool = obj.stakePool;
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
  constructor(obj: {}) {
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
export class createStakeAccountInstruction {
  tag: number;
  nonce: number;
  owner: number;
  stakePool: number;
  static schema: Schema = new Map([
    [
      createStakeAccountInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["nonce", "u8"],
          ["owner", "u8"],
          ["stakePool", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: { nonce: number; owner: number; stakePool: number }) {
    this.tag = 2;
    this.nonce = obj.nonce;
    this.owner = obj.owner;
    this.stakePool = obj.stakePool;
  }
  serialize(): Uint8Array {
    return serialize(createStakeAccountInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakeAccount: PublicKey,
    systemProgram: PublicKey,
    feePayer: PublicKey,
    rentSysvarAccount: PublicKey
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
      pubkey: feePayer,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: rentSysvarAccount,
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
      isWritable: true,
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
export class createStakePoolInstruction {
  tag: number;
  nonce: number;
  name: string;
  owner: number;
  destination: number;
  static schema: Schema = new Map([
    [
      createStakePoolInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["nonce", "u8"],
          ["name", "string"],
          ["owner", "u8"],
          ["destination", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    nonce: number;
    name: string;
    owner: number;
    destination: number;
  }) {
    this.tag = 1;
    this.nonce = obj.nonce;
    this.name = obj.name;
    this.owner = obj.owner;
    this.destination = obj.destination;
  }
  serialize(): Uint8Array {
    return serialize(createStakePoolInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePoolAccount: PublicKey,
    systemProgram: PublicKey,
    feePayer: PublicKey,
    rentSysvarAccount: PublicKey,
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
      pubkey: rentSysvarAccount,
      isSigner: false,
      isWritable: false,
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
