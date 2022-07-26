import { PublicKey, AccountInfo } from "@solana/web3.js";
import { StakeAccount, StakePool } from "@access-protocol";

import BN from 'bn.js';

export const getKey = (programId: string): Promise<[PublicKey, number]> => {
  return Promise.resolve([new PublicKey(programId), 1]);
};

export const stakeAccountRetrieve = (ownerKey: string, stakePoolKey: string, poolMinimumAtCreation: number, stakeAmount: number) => {
  return Promise.resolve(new StakeAccount({ 
    tag: 1, 
    owner: (new PublicKey(ownerKey)).toBytes(), 
    stakeAmount: new BN(stakeAmount), 
    stakePool:  (new PublicKey(stakePoolKey)).toBytes(),  
    lastClaimedTime: new BN(0),
    poolMinimumAtCreation: new BN(poolMinimumAtCreation),
    pendingUnstakeRequests: 0,
    unstakeRequests: []
  }));
};

export const stakePoolRetrieve = (ownerKey: string, minimumStakeAmount: number) => {
  return Promise.resolve(new StakePool({
    tag: 1,
    nonce: 1,
    currentDayIdx: 1,
    _padding: new Uint8Array(1),
    minimumStakeAmount: new BN(minimumStakeAmount),
    totalStaked: new BN(0),
    totalStakedLastCrank: new BN(0),
    lastCrankTime: new BN(1),
    lastClaimedTime: new BN(1),
    stakersPart: new BN(0),
    unstakePeriod: new BN(0),
    owner: (new PublicKey(ownerKey)).toBytes(),
    vault: new Uint8Array(1),
    balances: []
    }));
};

export const getBondAccount = (ownerKey: string) => {
  return {
    pubkey: new PublicKey(ownerKey),
    account: <AccountInfo<Buffer>> {
      executable: true,
      owner: new PublicKey(ownerKey),
      lamports: 1,
      data: new Buffer("")
    }
  };
}

export const bondAccountDeserialize = (ownerKey: string, stakePoolKey: string, totalStaked: number) => {
  return {
    tag: 1,
    owner: new PublicKey(ownerKey),
    totalAmountSold: new BN(1),
    totalStaked: new BN(totalStaked),
    totalQuoteAmount: new BN(1),
    quoteMint: new PublicKey(ownerKey),
    sellerTokenAccount: new PublicKey(ownerKey),
    unlockStartDate: new BN(1),
    unlockPeriod: new BN(1),
    unlockAmount: new BN(1),
    lastUnlockTime: new BN(1),
    totalUnlockedAmount: new BN(1),
    poolMinimumAtCreation: new BN(1),
    stakePool: new PublicKey(stakePoolKey),
    lastClaimedTime: new BN(1),
    sellers: []
  };
};
