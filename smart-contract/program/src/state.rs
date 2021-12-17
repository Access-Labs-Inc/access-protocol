use crate::error::MediaError;
use bonfida_utils::BorshSize;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{from_bytes_mut, try_cast_slice_mut, Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use std::cell::RefMut;
use std::mem::size_of;

// Just a random mint for now
pub const MEDIA_MINT: Pubkey =
    solana_program::pubkey!("EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp");

pub const SECONDS_IN_DAY: u64 = 3600 * 24;

pub const STAKER_MULTIPLIER: u64 = 80;
pub const OWNER_MULTIPLIER: u64 = 100 - STAKER_MULTIPLIER;
pub const STAKE_BUFFER_LEN: u64 = 365;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum Tag {
    Uninitialized,
    StakePool,
    StakeAccount,
    CentralState,
    Deleted,
}

impl BorshSize for Tag {
    fn borsh_len(&self) -> usize {
        1
    }
}

// TODO add total number of stakers?
#[derive(BorshSerialize, BorshDeserialize, BorshSize, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct StakePoolHeader {
    // Tag
    pub tag: u8,

    // Stake pool nonce
    pub nonce: u8,

    // Updated by a trustless cranker
    pub current_day_idx: u16,

    // Padding
    pub _padding: [u8; 4],

    // Total amount staked in the pool
    pub total_staked: u64,

    // Last unix timestamp when rewards were paid to the pool owner
    // through a permissionless crank
    pub last_crank_time: i64,

    // Last time the stake pool owner claimed
    pub last_claimed_time: i64,

    // Owner of the stake pool
    pub owner: [u8; 32],

    // Address to which rewards are sent
    pub rewards_destination: [u8; 32],

    // Stake pool vault
    pub vault: [u8; 32],
}

pub struct StakePool<'a> {
    pub header: RefMut<'a, StakePoolHeader>,
    pub balances: RefMut<'a, [u128]>, // of length STAKE_BUFFER_LEN
}

impl<'a> StakePool<'a> {
    pub fn get_checked<'b: 'a>(account_info: &'a AccountInfo<'b>) -> Result<Self, ProgramError> {
        let (header, balances) = RefMut::map_split(account_info.data.borrow_mut(), |s| {
            let (hd, rem) = s.split_at_mut(size_of::<StakePoolHeader>());
            (
                from_bytes_mut::<StakePoolHeader>(hd),
                try_cast_slice_mut(rem).unwrap(),
            )
        });

        if header.tag != Tag::StakePool as u8 && header.tag != Tag::Uninitialized as u8 {
            return Err(MediaError::DataTypeMismatch.into());
        }

        Ok(StakePool { header, balances })
    }

    pub fn push_balances_buff(&mut self, val: u128) {
        self.balances[((self.header.current_day_idx as u64) % STAKE_BUFFER_LEN) as usize] = val;
        self.header.current_day_idx += 1;
    }

    pub fn create_key(
        nonce: &u8,
        owner: &Pubkey,
        destination: &Pubkey,
        program_id: &Pubkey,
    ) -> Pubkey {
        let seeds: &[&[u8]] = &[
            StakePoolHeader::SEED.as_bytes(),
            &owner.to_bytes(),
            &destination.to_bytes(),
            &[*nonce],
        ];
        Pubkey::create_program_address(seeds, program_id).unwrap()
    }
}

impl StakePoolHeader {
    pub const SEED: &'static str = "stake_pool";

    pub fn new(owner: Pubkey, rewards_destination: Pubkey, nonce: u8, vault: Pubkey) -> Self {
        Self {
            tag: Tag::StakePool as u8,
            total_staked: 0,
            current_day_idx: 0,
            _padding: [0; 4],
            last_crank_time: Clock::get().unwrap().unix_timestamp,
            last_claimed_time: Clock::get().unwrap().unix_timestamp,
            owner: owner.to_bytes(),
            rewards_destination: rewards_destination.to_bytes(),
            nonce,
            vault: vault.to_bytes(),
        }
    }

    // pub fn save(&self, mut dst: &mut [u8]) {
    //     self.serialize(&mut dst).unwrap()
    // }

    // pub fn from_account_info(a: &AccountInfo) -> Result<StakePool, ProgramError> {
    //     let mut data = &a.data.borrow() as &[u8];
    //     if data[0] != Tag::StakePool as u8 && data[0] != Tag::Uninitialized as u8 {
    //         return Err(MediaError::DataTypeMismatch.into());
    //     }
    //     let result = StakePool::deserialize(&mut data)?;
    //     Ok(result)
    // }

    pub fn close(&mut self) {
        self.tag = Tag::Deleted as u8
    }

    pub fn deposit(&mut self, amount: u64) -> ProgramResult {
        self.total_staked = self.total_staked.checked_add(amount).unwrap();
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> ProgramResult {
        self.total_staked = self.total_staked.checked_sub(amount).unwrap();
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, BorshSize)]
pub struct StakeAccount {
    // Tag
    pub tag: Tag,

    // Owner of the stake account
    pub owner: Pubkey,

    // Amount staked in the account
    pub stake_amount: u64,

    // Stake pool to which the account belongs to
    pub stake_pool: Pubkey,

    // Last unix timestamp where rewards were claimed
    pub last_claimed_time: i64,
}

impl StakeAccount {
    pub const SEED: &'static str = "stake_account";

    pub fn new(owner: Pubkey, stake_pool: Pubkey, current_time: i64) -> Self {
        Self {
            tag: Tag::StakeAccount,
            owner,
            stake_amount: 0,
            stake_pool,
            last_claimed_time: current_time,
        }
    }

    pub fn create_key(
        nonce: &u8,
        owner: &Pubkey,
        stake_pool: &Pubkey,
        program_id: &Pubkey,
    ) -> Pubkey {
        let seeds: &[&[u8]] = &[
            StakeAccount::SEED.as_bytes(),
            &owner.to_bytes(),
            &stake_pool.to_bytes(),
            &[*nonce],
        ];
        Pubkey::create_program_address(seeds, program_id).unwrap()
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(a: &AccountInfo) -> Result<StakeAccount, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != Tag::StakeAccount as u8 && data[0] != Tag::Uninitialized as u8 {
            return Err(MediaError::DataTypeMismatch.into());
        }
        let result = StakeAccount::deserialize(&mut data)?;
        Ok(result)
    }

    pub fn close(&mut self) {
        self.tag = Tag::Deleted
    }

    pub fn deposit(&mut self, amount: u64) -> ProgramResult {
        self.stake_amount = self.stake_amount.checked_add(amount).unwrap();
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> ProgramResult {
        self.stake_amount = self.stake_amount.checked_sub(amount).unwrap();
        Ok(())
    }
}
#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, BorshSize)]
pub struct CentralState {
    // Tag
    pub tag: Tag,

    // Central state nonce
    pub signer_nonce: u8,

    // Daily inflation in token amount, inflation is paid from
    // the reserve owned by the central state
    pub daily_inflation: u64,

    // Central vault
    // From where the inflation is emitted
    pub central_vault: Pubkey,

    // Mint of the token being emitted
    pub token_mint: Pubkey,

    // Authority
    // The public key that can change the inflation
    pub authority: Pubkey,
}

impl CentralState {
    pub fn new(
        signer_nonce: u8,
        daily_inflation: u64,
        central_vault: Pubkey,
        token_mint: Pubkey,
        authority: Pubkey,
    ) -> Self {
        Self {
            tag: Tag::CentralState,
            signer_nonce,
            daily_inflation,
            central_vault,
            token_mint,
            authority,
        }
    }

    pub fn create_key(signer_nonce: &u8, program_id: &Pubkey) -> Pubkey {
        let signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[*signer_nonce]];
        Pubkey::create_program_address(signer_seeds, program_id).unwrap()
    }

    pub fn find_key(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&program_id.to_bytes()], program_id)
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(a: &AccountInfo) -> Result<CentralState, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != Tag::CentralState as u8 && data[0] != Tag::Uninitialized as u8 {
            return Err(MediaError::DataTypeMismatch.into());
        }
        let result = CentralState::deserialize(&mut data)?;
        Ok(result)
    }
}
