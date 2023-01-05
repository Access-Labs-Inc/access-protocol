// This file is auto-generated. DO NOT EDIT
import BN from "bn.js";
import { Schema, serialize } from "borsh";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}
export class adminFreezeInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      adminFreezeInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 20;
  }
  serialize(): Uint8Array {
    return serialize(adminFreezeInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    authority: PublicKey,
    accountToFreeze: PublicKey,
    centralState: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: accountToFreeze,
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
    this.tag = 19;
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
export class activateStakePoolInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      activateStakePoolInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 2;
  }
  serialize(): Uint8Array {
    return serialize(activateStakePoolInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    authority: PublicKey,
    stakePool: PublicKey,
    centralState: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: false,
    });
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
export class changeCentralStateAuthorityInstruction {
  tag: number;
  newAuthority: Uint8Array;
  static schema: Schema = new Map([
    [
      changeCentralStateAuthorityInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["newAuthority", [32]],
        ],
      },
    ],
  ]);
  constructor(obj: { newAuthority: Uint8Array }) {
    this.tag = 22;
    this.newAuthority = obj.newAuthority;
  }
  serialize(): Uint8Array {
    return serialize(changeCentralStateAuthorityInstruction.schema, this);
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
    this.tag = 12;
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
    this.tag = 18;
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
    this.tag = 16;
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
    stakePool: PublicKey,
    accessMint: PublicKey,
    poolVault: PublicKey,
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
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: accessMint,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: poolVault,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
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
    this.tag = 11;
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
    this.tag = 10;
  }
  serialize(): Uint8Array {
    return serialize(closeStakePoolInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    stakePoolAccount: PublicKey,
    poolVault: PublicKey,
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
      pubkey: poolVault,
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
    this.tag = 9;
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
    sellerIndex: BN;
  }) {
    this.tag = 13;
    this.buyer = obj.buyer;
    this.totalAmountSold = obj.totalAmountSold;
    this.totalQuoteAmount = obj.totalQuoteAmount;
    this.quoteMint = obj.quoteMint;
    this.sellerTokenAccount = obj.sellerTokenAccount;
    this.unlockStartDate = obj.unlockStartDate.fromTwos(64);
    this.unlockPeriod = obj.unlockPeriod.fromTwos(64);
    this.unlockAmount = obj.unlockAmount;
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
    this.tag = 3;
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
export class createStakePoolInstruction {
  tag: number;
  owner: Uint8Array;
  minimumStakeAmount: BN;
  static schema: Schema = new Map([
    [
      createStakePoolInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["owner", [32]],
          ["minimumStakeAmount", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { owner: Uint8Array; minimumStakeAmount: BN }) {
    this.tag = 1;
    this.owner = obj.owner;
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
export class executeUnstakeInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      executeUnstakeInstruction,
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
    return serialize(executeUnstakeInstruction.schema, this);
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
    this.tag = 14;
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
export class stakeInstruction {
  tag: number;
  amount: BN;
  hasBondAccount: boolean;
  has
  static schema: Schema = new Map([
    [
      stakeInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["amount", "u64"],
          ["has_bond_account", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    amount: BN
    hasBondAccount: boolean
  }) {
    this.tag = 4;
    this.amount = obj.amount;
    this.hasBondAccount = obj.hasBondAccount;
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
    vault: PublicKey,
    feeAccount: PublicKey,
    bondAccount: PublicKey | undefined,
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
    keys.push({
      pubkey: feeAccount,
      isSigner: false,
      isWritable: true,
    });
    if (bondAccount) {
      keys.push({
        pubkey: bondAccount,
        isSigner: false,
        isWritable: false,
      });
    }
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
    this.tag = 15;
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
    stakePool: PublicKey,
    poolVault: PublicKey,
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
      isWritable: false,
    });
    keys.push({
      pubkey: accessTokenDestination,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: stakePool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: poolVault,
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
  hasBondAccount: boolean;
  static schema: Schema = new Map([
    [
      unstakeInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["amount", "u64"],
          ["has_bond_account", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    amount: BN,
    hasBondAccount: boolean
  }) {
    this.tag = 5;
    this.amount = obj.amount;
    this.hasBondAccount = obj.hasBondAccount;
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
    bondAccount: PublicKey | undefined,
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
    if (bondAccount) {
      keys.push({
        pubkey: bondAccount,
        isSigner: false,
        isWritable: false,
      });
    }
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
export class changePoolMultiplierInstruction {
  tag: number;
  newMultiplier: BN;
  static schema: Schema = new Map([
    [
      changePoolMultiplierInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["newMultiplier", "u64"],
        ],
      },
    ],
  ]);
  constructor(obj: { newMultiplier: BN }) {
    this.tag = 21;
    this.newMultiplier = obj.newMultiplier;
  }
  serialize(): Uint8Array {
    return serialize(changePoolMultiplierInstruction.schema, this);
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
export class createCentralStateInstruction {
  tag: number;
  dailyInflation: BN;
  authority: Uint8Array;
  name: string;
  symbol: string;
  uri: string;
  static schema: Schema = new Map([
    [
      createCentralStateInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["dailyInflation", "u64"],
          ["authority", [32]],
          ["name", "string"],
          ["symbol", "string"],
          ["uri", "string"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    dailyInflation: BN;
    authority: Uint8Array;
    name: string;
    symbol: string;
    uri: string;
  }) {
    this.tag = 0;
    this.dailyInflation = obj.dailyInflation;
    this.authority = obj.authority;
    this.name = obj.name;
    this.symbol = obj.symbol;
    this.uri = obj.uri;
  }
  serialize(): Uint8Array {
    return serialize(createCentralStateInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    systemProgram: PublicKey,
    feePayer: PublicKey,
    mint: PublicKey,
    metadata: PublicKey,
    metadataProgram: PublicKey,
    rentSysvar: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralState,
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
    keys.push({
      pubkey: metadata,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: metadataProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: rentSysvar,
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
export class editMetadataInstruction {
  tag: number;
  name: string;
  symbol: string;
  uri: string;
  static schema: Schema = new Map([
    [
      editMetadataInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["name", "string"],
          ["symbol", "string"],
          ["uri", "string"],
        ],
      },
    ],
  ]);
  constructor(obj: { name: string; symbol: string; uri: string }) {
    this.tag = 23;
    this.name = obj.name;
    this.symbol = obj.symbol;
    this.uri = obj.uri;
  }
  serialize(): Uint8Array {
    return serialize(editMetadataInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    authority: PublicKey,
    metadata: PublicKey,
    metadataProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: metadata,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: metadataProgram,
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
    this.tag = 17;
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
    this.tag = 7;
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
export class claimRewardsInstruction {
  tag: number;
  allowZeroRewards: number;
  static schema: Schema = new Map([
    [
      claimRewardsInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["allowZeroRewards", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: { allowZeroRewards: number }) {
    this.tag = 8;
    this.allowZeroRewards = obj.allowZeroRewards;
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
