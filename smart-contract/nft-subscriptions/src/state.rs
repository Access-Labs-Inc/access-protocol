use crate::error::AccessError;
use bonfida_utils::BorshSize;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast_slice, from_bytes, from_bytes_mut, try_cast_slice_mut, Pod, Zeroable};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use std::cell::RefMut;
use std::convert::TryInto;
use std::mem::size_of;
use std::ops::DerefMut;

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq, FromPrimitive, ToPrimitive, Debug)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    SubscriptionMint,
}

#[derive(BorshSerialize, BorshDeserialize, BorshSize, Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C)]
#[allow(missing_docs)]
pub struct SubscriptionMint {
    /// Tag
    pub tag: u8,

    /// Stake pool nonce
    pub nonce: u8,

    /// Updated by a trustless cranker
    pub current_day_idx: u16,

    /// Padding
    pub _padding: [u8; 4],

    /// Minimum amount to stake to get access to the pool
    pub minimum_stake_amount: u64,

    /// Total amount staked in the pool
    pub total_staked: u64,

    /// Last time the stake pool owner claimed as an offset from the central state's creation time
    pub last_claimed_offset: u64,

    /// The % of pool rewards going to stakers
    pub stakers_part: u64,

    /// Owner of the stake pool
    pub owner: [u8; 32],

    /// Stake pool vault
    pub vault: [u8; 32],
}