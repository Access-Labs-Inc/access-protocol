import * as accessProtocolApi from "@access-protocol";
import { StakeAccount, StakePool, BondAccount } from "@access-protocol";
import { getKey, stakeAccountRetrieve, stakePoolRetrieve, getBondAccount, bondAccountDeserialize } from './stake.mocks';

const ownerKey = "DKaL649hhfHJqyv7P7SdudDdkbUPJ2VNnCdYz9B5tMe";
const stakePoolKey = "8Ra4Exdzb7Rubb5yy2j2R37usgjvEi75PPGHQ8PMG7zx";
const PROGRAM_ID = "GHPLV6HpsqbU5QXcH3Ps1vPvh8HHraX6PXiSjWZRfHwC";
process.env = {
  STAKE_POOL_KEY: stakePoolKey,
  RPC_URL: "https://example.com"
}

import { checkStake } from "../src/utils/stake"


// Enables us to mock functions from the @access-protocol module
jest.mock("@access-protocol", () => {
  return {
    __esModule: true,
    ...jest.requireActual("@access-protocol")
  };
});

beforeAll(() => {
  jest.spyOn(StakeAccount, "getKey")
  .mockReturnValue(getKey(PROGRAM_ID));

  jest.spyOn(StakePool, "retrieve")
  .mockReturnValue(
    stakePoolRetrieve(ownerKey, 1000)
  );
  
  jest.spyOn(accessProtocolApi, "getBondAccounts")
  .mockReturnValue(
    Promise.resolve([
      getBondAccount(ownerKey)
    ])
  );
});

describe("checkStake tests", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  test("Should be staked with Stake Account only", async() => {
    jest.spyOn(StakeAccount, "retrieve")
    .mockReturnValue(
      stakeAccountRetrieve(ownerKey, stakePoolKey, 1000, 1001)
    );

    jest.spyOn(BondAccount, "deserialize")
    .mockReturnValue(
      bondAccountDeserialize(ownerKey, stakePoolKey, 0)
    ); 

    expect(await checkStake(ownerKey)).toBeTruthy();
  });

  test("Should be staked with Stake Account and Bond Account combined", async() => {
    jest.spyOn(StakeAccount, "retrieve")
    .mockReturnValue(
      stakeAccountRetrieve(ownerKey, stakePoolKey, 1000, 400)
    );

    jest.spyOn(BondAccount, "deserialize")
    .mockReturnValue(
      bondAccountDeserialize(ownerKey, stakePoolKey, 700)
    ); 

    expect(await checkStake(ownerKey)).toBeTruthy();
  });

  test("Should not be staked", async() => {
    jest.spyOn(StakeAccount, "retrieve")
    .mockReturnValue(
      stakeAccountRetrieve(ownerKey, stakePoolKey, 1000, 400)
    );

    jest.spyOn(BondAccount, "deserialize")
    .mockReturnValue(
      bondAccountDeserialize(ownerKey, stakePoolKey, 300)
    ); 

    expect(await checkStake(ownerKey)).toBeFalsy();
  });

  test("Should not be staked because Bond Pool is not linked to Stake Pool", async() => {
    jest.spyOn(StakeAccount, "retrieve")
    .mockReturnValue(
      stakeAccountRetrieve(ownerKey, stakePoolKey, 1000, 400)
    );

    jest.spyOn(BondAccount, "deserialize")
    .mockReturnValue(
      bondAccountDeserialize(ownerKey, "6co5dnCAnbydVWUi81zhdQgCHezmCXZQzATUTTEdjU44", 900)
    ); 

    expect(await checkStake(ownerKey)).toBeFalsy();
  });

  test("Should be staked with accumulated stake value from multiple linked bond accounts", async() => {
    jest.spyOn(StakeAccount, "retrieve")
    .mockReturnValue(
      stakeAccountRetrieve(ownerKey, stakePoolKey, 1000, 101)
    );

    jest.spyOn(BondAccount, "deserialize")
    .mockReturnValue(
      bondAccountDeserialize(ownerKey, "6co5dnCAnbydVWUi81zhdQgCHezmCXZQzATUTTEdjU44", 300)
    )
    .mockReturnValueOnce(
      bondAccountDeserialize(ownerKey, stakePoolKey, 300)
    )
    .mockReturnValueOnce(
      bondAccountDeserialize(ownerKey, stakePoolKey, 300)
    )
    .mockReturnValueOnce(
      bondAccountDeserialize(ownerKey, stakePoolKey, 300)
    )
    .mockReturnValueOnce(
      bondAccountDeserialize(ownerKey, "6co5dnCAnbydVWUi81zhdQgCHezmCXZQzATUTTEdjU44", 300)
    );

    jest.spyOn(accessProtocolApi, "getBondAccounts")
    .mockReturnValue(
      Promise.resolve([
        getBondAccount(ownerKey),
        getBondAccount(ownerKey),
        getBondAccount(ownerKey),
        getBondAccount(ownerKey),
        getBondAccount(ownerKey)
      ])
    );

    expect(await checkStake(ownerKey)).toBeTruthy();
  });
});
