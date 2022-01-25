import { Connection, PublicKey, MemcmpFilter } from "@solana/web3.js";
import { ACCESS_PROGRAM_ID } from "./bindings";

/**
 * This function can be used to find all stake accounts of a user
 * @param connection The Solana RPC connection
 * @param owner The owner of the stake accounts to retrieve
 * @returns
 */
export const getStakeAccounts = async (
  connection: Connection,
  owner: PublicKey
) => {
  const filters: MemcmpFilter[] = [
    {
      memcmp: {
        offset: 0,
        bytes: "3",
      },
    },
    {
      memcmp: {
        offset: 1,
        bytes: owner.toBase58(),
      },
    },
  ];
  return await connection.getProgramAccounts(ACCESS_PROGRAM_ID, {
    filters,
  });
};

/**
 * This function can be used to find all stake pools of a user
 * @param connection The Solana RPC connection
 * @param owner The owner of the stake pools to retrieve
 * @returns
 */
export const getStakePools = async (
  connection: Connection,
  owner: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "2",
      },
    },
    {
      memcmp: {
        offset: 1 + 1 + 2 + 4 + 8 + 8 + 8,
        bytes: owner.toBase58(),
      },
    },
  ];
  return await connection.getProgramAccounts(ACCESS_PROGRAM_ID, {
    filters,
  });
};

/**
 * This function can be used to find all bonds of a user
 * @param connection The Solana RPC connection
 * @param owner The owner of the bonds to retrieve
 * @returns
 */
export const getBondAccounts = async (
  connection: Connection,
  owner: PublicKey
) => {
  const filters = [
    {
      memcmp: {
        offset: 0,
        bytes: "5",
      },
    },
    {
      memcmp: {
        offset: 1,
        bytes: owner.toBase58(),
      },
    },
  ];
  return await connection.getProgramAccounts(ACCESS_PROGRAM_ID, {
    filters,
  });
};
