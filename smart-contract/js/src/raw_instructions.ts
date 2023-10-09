// This file is auto-generated. DO NOT EDIT
import * as BN from "bn.js";
import { Schema, serialize } from "borsh";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { FeeRecipient } from "./state";

interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}

export interface TaggedInstruction {
  tag: number;
}

export class adminSetupFeeSplitInstruction implements TaggedInstruction {
  tag: number;
  recipients: FeeRecipient[];
  static schema: Schema = new Map<any, any>([
    [
      adminSetupFeeSplitInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["recipients", [FeeRecipient]],
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

  constructor(obj: { recipients: FeeRecipient[] }) {
    this.tag = 27;
    this.recipients = obj.recipients;
  }

  serialize(): Uint8Array {
    return serialize(adminSetupFeeSplitInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    authority: PublicKey,
    centralState: PublicKey,
    systemProgram: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: false,
    });
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
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class migrateCentralStateV2Instruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      migrateCentralStateV2Instruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 30;
  }

  serialize(): Uint8Array {
    return serialize(migrateCentralStateV2Instruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    systemProgram: PublicKey,
    feePayer: PublicKey,
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
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class closeStakePoolInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      closeStakePoolInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 9;
  }

  serialize(): Uint8Array {
    return serialize(closeStakePoolInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    stakePoolAccount: PublicKey,
    poolVault: PublicKey,
    owner: PublicKey,
    centralState: PublicKey,
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

export class changeInflationInstruction implements TaggedInstruction {
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

  constructor(obj: {
    dailyInflation: BN;
  }) {
    this.tag = 11;
    this.dailyInflation = obj.dailyInflation;
  }

  serialize(): Uint8Array {
    return serialize(changeInflationInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    authority: PublicKey,
    mint: PublicKey,
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

export class unlockBondTokensInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      unlockBondTokensInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 14;
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
    splTokenProgram: PublicKey,
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

export class adminSetProtocolFeeInstruction implements TaggedInstruction {
  tag: number;
  protocolFeeBasisPoints: number;
  static schema: Schema = new Map([
    [
      adminSetProtocolFeeInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["protocolFeeBasisPoints", "u16"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    protocolFeeBasisPoints: number;
  }) {
    this.tag = 29;
    this.protocolFeeBasisPoints = obj.protocolFeeBasisPoints;
  }

  serialize(): Uint8Array {
    return serialize(adminSetProtocolFeeInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    authority: PublicKey,
    centralState: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: authority,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: centralState,
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

export class adminFreezeInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      adminFreezeInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 19;
  }

  serialize(): Uint8Array {
    return serialize(adminFreezeInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    authority: PublicKey,
    accountToFreeze: PublicKey,
    centralState: PublicKey,
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

export class adminRenounceInstruction implements TaggedInstruction {
  tag: number;
  ix: number;
  static schema: Schema = new Map([
    [
      adminRenounceInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["ix", "u8"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    ix: number;
  }) {
    this.tag = 32;
    this.ix = obj.ix;
  }

  serialize(): Uint8Array {
    return serialize(adminRenounceInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    authority: PublicKey,
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

export class changePoolMinimumInstruction implements TaggedInstruction {
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

  constructor(obj: {
    newMinimum: BN;
  }) {
    this.tag = 17;
    this.newMinimum = obj.newMinimum;
  }

  serialize(): Uint8Array {
    return serialize(changePoolMinimumInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    stakePool: PublicKey,
    stakePoolOwner: PublicKey,
    centralState: PublicKey,
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

export class createBondV2Instruction implements TaggedInstruction {
  tag: number;
  amount: BN;
  unlockTimestamp: BN | null;
  static schema: Schema = new Map([
    [
      createBondV2Instruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["amount", "u64"],
          ["unlockTimestamp", { kind: "option", type: "u64" }],
        ],
      },
    ],
  ]);

  constructor(obj: {
    amount: BN;
    unlockTimestamp: BN | null;
  }) {
    this.tag = 23;
    this.amount = obj.amount;
    this.unlockTimestamp = obj.unlockTimestamp;
  }

  serialize(): Uint8Array {
    return serialize(createBondV2Instruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    feePayer: PublicKey,
    from: PublicKey,
    fromAta: PublicKey,
    to: PublicKey,
    bondV2Account: PublicKey,
    centralState: PublicKey,
    centralStateVault: PublicKey,
    pool: PublicKey,
    poolVault: PublicKey,
    mint: PublicKey,
    splTokenProgram: PublicKey,
    systemProgram: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: feePayer,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: from,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: fromAta,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: to,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: bondV2Account,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralStateVault,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: pool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: poolVault,
      isSigner: false,
      isWritable: true,
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
    keys.push({
      pubkey: systemProgram,
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

export class claimPoolRewardsInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      claimPoolRewardsInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 6;
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
    splTokenProgram: PublicKey,
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

export class signBondInstruction implements TaggedInstruction {
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

  constructor(obj: {
    sellerIndex: BN;
  }) {
    this.tag = 13;
    this.sellerIndex = obj.sellerIndex;
  }

  serialize(): Uint8Array {
    return serialize(signBondInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    seller: PublicKey,
    bondAccount: PublicKey,
    centralState: PublicKey,
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
    this.tag = 7;
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
    const keys: AccountKey[] = [];
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

export class distributeFeesInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      distributeFeesInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 28;
  }

  serialize(): Uint8Array {
    return serialize(distributeFeesInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    centralStateVault: PublicKey,
    splTokenProgram: PublicKey,
    mint: PublicKey,
    tokenAccounts: PublicKey[],
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralStateVault,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: mint,
      isSigner: false,
      isWritable: true,
    });
    for (let k of tokenAccounts) {
      keys.push({
        pubkey: k,
        isSigner: false,
        isWritable: true,
      });
    }
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class adminProgramFreezeInstruction implements TaggedInstruction {
  tag: number;
  ixGate: BN;
  static schema: Schema = new Map([
    [
      adminProgramFreezeInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["ixGate", "u128"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    ixGate: BN;
  }) {
    this.tag = 31;
    this.ixGate = obj.ixGate;
  }

  serialize(): Uint8Array {
    return serialize(adminProgramFreezeInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    authority: PublicKey,
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

export class stakeInstruction implements TaggedInstruction {
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

  constructor(obj: {
    amount: BN;
  }) {
    this.tag = 4;
    this.amount = obj.amount;
  }

  serialize(): Uint8Array {
    return serialize(stakeInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    stakeAccount: PublicKey,
    stakePool: PublicKey,
    owner: PublicKey,
    sourceToken: PublicKey,
    splTokenProgram: PublicKey,
    vault: PublicKey,
    centralStateVault: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralState,
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
      pubkey: centralStateVault,
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

export class createCentralStateInstruction implements TaggedInstruction {
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

  constructor(obj: {
    dailyInflation: BN;
    authority: Uint8Array;
  }) {
    this.tag = 0;
    this.dailyInflation = obj.dailyInflation;
    this.authority = obj.authority;
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
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class claimBondV2RewardsInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      claimBondV2RewardsInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 25;
  }

  serialize(): Uint8Array {
    return serialize(claimBondV2RewardsInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    pool: PublicKey,
    bondV2Account: PublicKey,
    owner: PublicKey,
    rewardsDestination: PublicKey,
    centralState: PublicKey,
    mint: PublicKey,
    splTokenProgram: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: pool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: bondV2Account,
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

export class changePoolMultiplierInstruction implements TaggedInstruction {
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

  constructor(obj: {
    newMultiplier: BN;
  }) {
    this.tag = 20;
    this.newMultiplier = obj.newMultiplier;
  }

  serialize(): Uint8Array {
    return serialize(changePoolMultiplierInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    stakePool: PublicKey,
    stakePoolOwner: PublicKey,
    centralState: PublicKey,
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

export class closeStakeAccountInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      closeStakeAccountInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 10;
  }

  serialize(): Uint8Array {
    return serialize(closeStakeAccountInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    stakeAccount: PublicKey,
    owner: PublicKey,
    centralState: PublicKey,
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

export class crankInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      crankInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 8;
  }

  serialize(): Uint8Array {
    return serialize(crankInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    stakePool: PublicKey,
    centralState: PublicKey,
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
      isWritable: true,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class addToBondV2Instruction implements TaggedInstruction {
  tag: number;
  amount: BN;
  static schema: Schema = new Map([
    [
      addToBondV2Instruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["amount", "u64"],
        ],
      },
    ],
  ]);

  constructor(obj: {
    amount: BN;
  }) {
    this.tag = 24;
    this.amount = obj.amount;
  }

  serialize(): Uint8Array {
    return serialize(addToBondV2Instruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    from: PublicKey,
    fromAta: PublicKey,
    bondV2Account: PublicKey,
    centralState: PublicKey,
    centralStateVault: PublicKey,
    pool: PublicKey,
    poolVault: PublicKey,
    mint: PublicKey,
    splTokenProgram: PublicKey,
    systemProgram: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: from,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: fromAta,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: bondV2Account,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: centralStateVault,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: pool,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: poolVault,
      isSigner: false,
      isWritable: true,
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
    keys.push({
      pubkey: systemProgram,
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

export class claimBondInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      claimBondInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 15;
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
    splTokenProgram: PublicKey,
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

export class editMetadataInstruction implements TaggedInstruction {
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

  constructor(obj: {
    name: string;
    symbol: string;
    uri: string;
  }) {
    this.tag = 22;
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
    metadataProgram: PublicKey,
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

export class createStakeAccountInstruction implements TaggedInstruction {
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

  constructor(obj: {
    nonce: number;
    owner: Uint8Array;
  }) {
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
    feePayer: PublicKey,
    centralState: PublicKey,
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

export class claimBondRewardsInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      claimBondRewardsInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 16;
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
    splTokenProgram: PublicKey,
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

export class createBondInstruction implements TaggedInstruction {
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
    this.tag = 12;
    this.buyer = obj.buyer;
    this.totalAmountSold = obj.totalAmountSold;
    this.totalQuoteAmount = obj.totalQuoteAmount;
    this.quoteMint = obj.quoteMint;
    this.sellerTokenAccount = obj.sellerTokenAccount;
    this.unlockStartDate = obj.unlockStartDate;
    this.unlockPeriod = obj.unlockPeriod;
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
    feePayer: PublicKey,
    centralState: PublicKey,
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

export class unstakeInstruction implements TaggedInstruction {
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

  constructor(obj: {
    amount: BN;
  }) {
    this.tag = 5;
    this.amount = obj.amount;
  }

  serialize(): Uint8Array {
    return serialize(unstakeInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    stakeAccount: PublicKey,
    stakePool: PublicKey,
    owner: PublicKey,
    destinationToken: PublicKey,
    splTokenProgram: PublicKey,
    vault: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralState,
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

export class createStakePoolInstruction implements TaggedInstruction {
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

  constructor(obj: {
    owner: Uint8Array;
    minimumStakeAmount: BN;
  }) {
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
    vault: PublicKey,
    centralState: PublicKey,
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

export class changeCentralStateAuthorityInstruction implements TaggedInstruction {
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

  constructor(obj: {
    newAuthority: Uint8Array;
  }) {
    this.tag = 21;
    this.newAuthority = obj.newAuthority;
  }

  serialize(): Uint8Array {
    return serialize(changeCentralStateAuthorityInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    authority: PublicKey,
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

export class unlockBondV2Instruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      unlockBondV2Instruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
      },
    ],
  ]);

  constructor() {
    this.tag = 26;
  }

  serialize(): Uint8Array {
    return serialize(unlockBondV2Instruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    centralState: PublicKey,
    bondV2Account: PublicKey,
    owner: PublicKey,
    ownerTokenAccount: PublicKey,
    pool: PublicKey,
    poolVault: PublicKey,
    splTokenProgram: PublicKey,
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: centralState,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: bondV2Account,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: owner,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: ownerTokenAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: pool,
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

export class adminMintInstruction implements TaggedInstruction {
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

  constructor(obj: {
    amount: BN;
  }) {
    this.tag = 18;
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
    splTokenProgram: PublicKey,
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

export class activateStakePoolInstruction implements TaggedInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      activateStakePoolInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
        ],
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
    stakePool: PublicKey,
    centralState: PublicKey,
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
