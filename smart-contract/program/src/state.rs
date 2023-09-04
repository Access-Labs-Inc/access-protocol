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

/// ACCESS token mint
pub const ACCESS_MINT: Pubkey =
    solana_program::pubkey!("5MAYDfq5yxtudAhtfyuMBuHZjgAbaS9tbEyEQYAhDS5y");

#[allow(missing_docs)]
pub const SECONDS_IN_DAY: u64 = if cfg!(feature = "days-to-sec-15m") {
    15 * 60
} else if cfg!(feature = "days-to-sec-10s") {
    10
} else {
    3600 * 24
};

/// Percentage of the staking rewards going to stakers
pub const STAKER_MULTIPLIER: u64 = 50;

/// Percentage of the staking rewards going to the pool owner
pub const OWNER_MULTIPLIER: u64 = 100 - STAKER_MULTIPLIER;

/// Length of the circular buffer (stores balances for 1 year)
pub const STAKE_BUFFER_LEN: u64 = 274; // 9 Months

/// Max pending unstake requests
pub const MAX_UNSTAKE_REQUEST: usize = 10;

/// Fees charged on staking instruction in % (i.e FEES = 1 <-> 1% fee charged)
pub const FEES: u64 = 2;

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq, FromPrimitive, ToPrimitive, Debug)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    StakePool,
    InactiveStakePool,
    StakeAccount,
    // Bond accounts are inactive until the buyer transfered the funds
    InactiveBondAccount,
    BondAccount,
    BondAccountV2,
    CentralState,
    Deleted,
    // Accounts frozen by the central state authority
    FrozenStakePool,
    FrozenStakeAccount,
    FrozenBondAccount,
    FrozenBondAccountV2, // todo use or delete
}

impl Tag {
    /// Freeze or unfreeze an account tag
    pub fn opposite(&self) -> Result<Tag, ProgramError> {
        let tag = match self {
            Tag::StakePool => Tag::FrozenStakePool,
            Tag::StakeAccount => Tag::FrozenStakeAccount,
            Tag::BondAccount => Tag::FrozenBondAccount,
            Tag::BondAccountV2 => Tag::FrozenBondAccountV2,
            Tag::FrozenStakePool => Tag::StakePool,
            Tag::FrozenStakeAccount => Tag::StakeAccount,
            Tag::FrozenBondAccount => Tag::BondAccount,
            Tag::FrozenBondAccountV2 => Tag::BondAccountV2,
            _ => return Err(AccessError::InvalidTagChange.into()),
        };

        Ok(tag)
    }
}

#[derive(BorshSerialize, BorshDeserialize, BorshSize, Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C)]
#[allow(missing_docs)]
pub struct StakePoolHeader {
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

#[allow(missing_docs)]
pub struct StakePool<H, B> {
    pub header: H,
    /// Circular buffer of length STAKE_BUFFER_LEN storing (inflation * pool_total_staked / total_staked) in FP32 format
    pub balances: B,
}

/// The Rewards structure that is held in the stake pools circular buffer.
/// The two fields represent the share that is owed to the pool owner and the stakers respectively.
/// The values are stored in the FP32 format.
#[derive(Pod, Clone, Copy, Zeroable, Debug)]
#[repr(C)]
pub struct RewardsTuple {
    pub(crate) pool_reward: u128,
    pub(crate) stakers_reward: u128,
}

#[allow(missing_docs)]
pub type StakePoolRef<'a> = StakePool<RefMut<'a, StakePoolHeader>, RefMut<'a, [RewardsTuple]>>;

#[allow(missing_docs)]
pub type StakePoolHeaped = StakePool<Box<StakePoolHeader>, Box<[RewardsTuple]>>;

#[allow(missing_docs)]
impl<'a> StakePoolRef<'a> {
    pub fn get_checked<'b: 'a>(
        account_info: &'a AccountInfo<'b>,
        allowed_tags: Vec<Tag>,
    ) -> Result<Self, ProgramError> {
        let (header, balances) = RefMut::map_split(account_info.data.borrow_mut(), |s| {
            let (hd, rem) = s.split_at_mut(size_of::<StakePoolHeader>());
            (
                from_bytes_mut::<StakePoolHeader>(hd),
                try_cast_slice_mut(rem).unwrap(),
            )
        });

        let tag = FromPrimitive::from_u8(header.tag).ok_or(ProgramError::InvalidAccountData)?;
        if !allowed_tags.contains(&tag) {
            return Err(AccessError::DataTypeMismatch.into());
        }

        Ok(StakePool { header, balances })
    }
}

#[allow(missing_docs)]
impl StakePoolHeaped {
    pub fn from_buffer(buf: &[u8]) -> Self {
        println!("StakePoolHeaped::from_buffer: buf.len() = {}", buf.len());
        let (header, balances) = buf.split_at(size_of::<StakePoolHeader>());
        println!("StakePoolHeaped::from_buffer: header.len() = {}", header.len());
        let header = from_bytes::<StakePoolHeader>(header);
        println!("StakePoolHeaped::from_buffer: header = {:?}", header);
        let balances = cast_slice::<_, RewardsTuple>(balances);
        println!("StakePoolHeaped::from_buffer: balances.len() = {}", balances.len());
        Self {
            header: Box::new(*header),
            balances: Box::from(balances),
        }
    }
}

#[allow(missing_docs)]
impl<H: DerefMut<Target = StakePoolHeader>, B: DerefMut<Target = [RewardsTuple]>> StakePool<H, B> {
    pub fn push_balances_buff(
        &mut self,
        current_offset: u64,
        rewards: RewardsTuple,
    ) -> Result<(), ProgramError> {
        let nb_days_passed = current_offset.checked_sub(self.header.current_day_idx as u64).ok_or(
            AccessError::Overflow,
        )?;
        for i in 1..nb_days_passed {
            self.balances[(((self.header.current_day_idx as u64)
                .checked_add(i)
                .ok_or(AccessError::Overflow)?)
                % STAKE_BUFFER_LEN) as usize] = RewardsTuple {
                pool_reward: 0,
                stakers_reward: 0,
            };
        }
        self.header.current_day_idx = self
            .header
            .current_day_idx
            .checked_add(
                nb_days_passed
                    .try_into()
                    .map_err(|_| AccessError::Overflow)?,
            )
            .ok_or(AccessError::Overflow)?;

        self.balances[(((self.header.current_day_idx - 1) as u64) % STAKE_BUFFER_LEN) as usize] =
            rewards;
        Ok(())
    }

    pub fn create_key(
        nonce: &u8,
        owner: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<Pubkey, ProgramError> {
        let seeds: &[&[u8]] = &[StakePoolHeader::SEED, &owner.to_bytes(), &[*nonce]];
        Pubkey::create_program_address(seeds, program_id).map_err(|_| ProgramError::InvalidSeeds)
    }
}

#[allow(missing_docs)]
impl StakePool<(), ()> {
    pub fn find_key(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[StakePoolHeader::SEED, &owner.to_bytes()];
        Pubkey::find_program_address(seeds, program_id)
    }
}

#[allow(missing_docs)]
impl StakePoolHeader {
    pub const SEED: &'static [u8; 10] = b"stake_pool";

    pub fn new(
        owner: Pubkey,
        nonce: u8,
        vault: Pubkey,
        minimum_stake_amount: u64,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            tag: Tag::InactiveStakePool as u8,
            total_staked: 0,
            current_day_idx: 0,
            _padding: [0; 4],
            last_claimed_offset: 0,
            owner: owner.to_bytes(),
            nonce,
            vault: vault.to_bytes(),
            minimum_stake_amount,
            stakers_part: STAKER_MULTIPLIER,
        })
    }

    pub fn close(&mut self) {
        self.tag = Tag::Deleted as u8
    }

    pub fn deposit(&mut self, amount: u64) -> ProgramResult {
        self.total_staked = self
            .total_staked
            .checked_add(amount)
            .ok_or(AccessError::Overflow)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> ProgramResult {
        self.total_staked = self
            .total_staked
            .checked_sub(amount)
            .ok_or(AccessError::Overflow)?;
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
#[allow(missing_docs)]
pub struct StakeAccount {
    /// Tag
    pub tag: Tag,

    /// Owner of the stake account
    pub owner: Pubkey,

    /// Amount staked in the account
    pub stake_amount: u64,

    /// Stake pool to which the account belongs to
    pub stake_pool: Pubkey,

    /// Offset of a last day where rewards were claimed from the contract creation date
    pub last_claimed_offset: u64,

    /// Minimum stakeable amount of the pool when the account
    /// was created
    pub pool_minimum_at_creation: u64,
}

#[allow(missing_docs)]
impl StakeAccount {
    pub const SEED: &'static [u8; 13] = b"stake_account";

    pub fn new(owner: Pubkey, stake_pool: Pubkey, pool_minimum_at_creation: u64) -> Self {
        Self {
            tag: Tag::StakeAccount,
            owner,
            stake_amount: 0,
            stake_pool,
            last_claimed_offset: 0,
            pool_minimum_at_creation,
        }
    }

    pub fn create_key(
        nonce: &u8,
        owner: &Pubkey,
        stake_pool: &Pubkey,
        program_id: &Pubkey,
    ) -> Result<Pubkey, ProgramError> {
        let seeds: &[&[u8]] = &[
            StakeAccount::SEED,
            &owner.to_bytes(),
            &stake_pool.to_bytes(),
            &[*nonce],
        ];
        Pubkey::create_program_address(seeds, program_id).map_err(|_| ProgramError::InvalidSeeds)
    }

    pub fn find_key(owner: &Pubkey, stake_pool: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[
            StakeAccount::SEED,
            &owner.to_bytes(),
            &stake_pool.to_bytes(),
        ];
        Pubkey::find_program_address(seeds, program_id)
    }

    pub fn save(&self, mut dst: &mut [u8]) -> ProgramResult {
        self.serialize(&mut dst)
            .map_err(|_| ProgramError::InvalidAccountData)
    }

    pub fn from_account_info(a: &AccountInfo) -> Result<StakeAccount, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != Tag::StakeAccount as u8 && data[0] != Tag::Uninitialized as u8 {
            return Err(AccessError::DataTypeMismatch.into());
        }
        let result = StakeAccount::deserialize(&mut data)?;
        Ok(result)
    }

    pub fn close(&mut self) {
        self.tag = Tag::Deleted
    }

    pub fn deposit(&mut self, amount: u64) -> ProgramResult {
        self.stake_amount = self
            .stake_amount
            .checked_add(amount)
            .ok_or(AccessError::Overflow)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> ProgramResult {
        self.stake_amount = self
            .stake_amount
            .checked_sub(amount)
            .ok_or(AccessError::Overflow)?;
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
#[allow(missing_docs)]
pub struct CentralState {
    /// Tag
    pub tag: Tag,

    /// Central state nonce
    pub signer_nonce: u8,

    /// Daily inflation in token amount, inflation is paid from
    /// the reserve owned by the central state
    pub daily_inflation: u64,

    /// Mint of the token being emitted
    pub token_mint: Pubkey,

    /// Authority
    /// The public key that can change the inflation
    pub authority: Pubkey,

    /// Creation timestamp
    pub creation_time: i64,

    /// Total amount of staked tokens
    pub total_staked: u64,

    /// The daily total_staked snapshot to calculate correctly calculate the pool rewards
    pub total_staked_snapshot: u64,

    /// The offset of the total_staked_snapshot from the creation_time in days
    pub last_snapshot_offset: u64,
}

impl CentralState {
    #[allow(missing_docs)]
    pub fn new(
        signer_nonce: u8,
        daily_inflation: u64,
        token_mint: Pubkey,
        authority: Pubkey,
        total_staked: u64,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            tag: Tag::CentralState,
            signer_nonce,
            daily_inflation,
            token_mint,
            authority,
            creation_time: Clock::get()?.unix_timestamp,
            total_staked,
            total_staked_snapshot: 0,
            last_snapshot_offset: 0,
        })
    }
    #[allow(missing_docs)]
    pub fn create_key(signer_nonce: &u8, program_id: &Pubkey) -> Result<Pubkey, ProgramError> {
        let signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[*signer_nonce]];
        Pubkey::create_program_address(signer_seeds, program_id)
            .map_err(|_| ProgramError::InvalidSeeds)
    }
    #[allow(missing_docs)]
    pub fn find_key(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&program_id.to_bytes()], program_id)
    }
    #[allow(missing_docs)]
    pub fn save(&self, mut dst: &mut [u8]) -> ProgramResult {
        self.serialize(&mut dst)
            .map_err(|_| ProgramError::InvalidAccountData)
    }
    #[allow(missing_docs)]
    pub fn from_account_info(a: &AccountInfo) -> Result<CentralState, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != Tag::CentralState as u8 && data[0] != Tag::Uninitialized as u8 {
            return Err(AccessError::DataTypeMismatch.into());
        }
        let result = CentralState::deserialize(&mut data)?;
        Ok(result)
    }
    #[allow(missing_docs)]
    pub fn get_current_offset(&self) -> Result<u64, ProgramError> {
        let current_time = Clock::get()?.unix_timestamp as u64;
        Ok((current_time - self.creation_time as u64) / SECONDS_IN_DAY)
    }
}

/// Number of sellers who need to agree for a bond to be sold
pub const BOND_SIGNER_THRESHOLD: u64 = 1;

/// List of authorized bond sellers
pub const AUTHORIZED_BOND_SELLERS: [Pubkey; 1] = [solana_program::pubkey!(
    "3Nrq6mCNL5i8Qk4APhggbwXismcsF23gNVDEaKycZBL8"
)];

#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
#[allow(missing_docs)]
pub struct BondAccount {
    // Tag
    pub tag: Tag,

    // Owner of the bond
    pub owner: Pubkey,

    // Total amount sold
    pub total_amount_sold: u64,

    // Total staked tokens
    pub total_staked: u64,

    // Total quote token
    pub total_quote_amount: u64,

    // Quote mint used to buy the bond
    pub quote_mint: Pubkey,

    // Seller token account (i.e destination of the quote tokens)
    pub seller_token_account: Pubkey,

    // Unlock start date
    pub unlock_start_date: i64,

    // Unlock period
    // time interval at which the tokens unlock
    pub unlock_period: i64,

    // Unlock amount
    // amount unlocked at every unlock_period
    pub unlock_amount: u64,

    // Last unlock date
    pub last_unlock_time: i64,

    // Total amount unlocked (metric)
    pub total_unlocked_amount: u64,

    // Minimum stakeable amount of the pool when the account
    // was created
    pub pool_minimum_at_creation: u64,

    // Stake pool to which the account belongs to
    pub stake_pool: Pubkey,

    // Last offset of the from the contract creation time in days
    pub last_claimed_offset: u64,

    // Sellers who signed for the sell of the bond account
    pub sellers: Vec<Pubkey>,
}

#[allow(missing_docs)]
impl BondAccount {
    pub const SEED: &'static [u8; 12] = b"bond_account";

    pub fn create_key(owner: &Pubkey, total_amount_sold: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[
            BondAccount::SEED,
            &owner.to_bytes(),
            &total_amount_sold.to_le_bytes(),
        ];
        Pubkey::find_program_address(seeds, program_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner: Pubkey,
        total_amount_sold: u64,
        total_quote_amount: u64,
        quote_mint: Pubkey,
        seller_token_account: Pubkey,
        unlock_start_date: i64,
        unlock_period: i64,
        unlock_amount: u64,
        last_unlock_time: i64,
        pool_minimum_at_creation: u64,
        stake_pool: Pubkey,
        seller: Pubkey,
    ) -> Self {
        let sellers = vec![seller];
        Self {
            tag: Tag::InactiveBondAccount,
            owner,
            total_amount_sold,
            total_staked: total_amount_sold,
            total_quote_amount,
            quote_mint,
            seller_token_account,
            unlock_start_date,
            unlock_period,
            unlock_amount,
            last_unlock_time,
            total_unlocked_amount: 0,
            stake_pool,
            last_claimed_offset: 0,
            sellers,
            pool_minimum_at_creation,
        }
    }

    pub fn save(&self, mut dst: &mut [u8]) -> ProgramResult {
        self.serialize(&mut dst)
            .map_err(|_| ProgramError::InvalidAccountData)
    }

    pub fn is_active(&self) -> bool {
        self.tag == Tag::BondAccount
    }

    pub fn activate(&mut self, current_offset: u64) -> ProgramResult {
        self.tag = Tag::BondAccount;
        self.last_claimed_offset = current_offset;
        let current_time = Clock::get()?.unix_timestamp;
        self.last_unlock_time = std::cmp::max(current_time, self.unlock_start_date);
        Ok(())
    }

    pub fn from_account_info(
        a: &AccountInfo,
        allow_inactive: bool,
    ) -> Result<BondAccount, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        let tag = if allow_inactive {
            Tag::InactiveBondAccount
        } else {
            Tag::BondAccount
        };
        if data[0] != tag as u8 && data[0] != Tag::Uninitialized as u8 {
            return Err(AccessError::DataTypeMismatch.into());
        }
        let result = BondAccount::deserialize(&mut data)?;
        Ok(result)
    }

    pub fn calc_unlock_amount(&self, missed_periods: u64) -> Result<u64, ProgramError> {
        msg!("Missed periods {}", missed_periods);
        let cumulated_unlock_amnt = (missed_periods)
            .checked_mul(self.unlock_amount)
            .ok_or(AccessError::Overflow)?;
        msg!(
            "Unlock amount {} Total amount {}",
            cumulated_unlock_amnt,
            self.total_amount_sold
        );

        Ok(std::cmp::min(
            cumulated_unlock_amnt,
            self.total_amount_sold
                .checked_sub(self.total_unlocked_amount)
                .ok_or(AccessError::Overflow)?,
        ))
    }
}


#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
#[allow(missing_docs)]
pub struct BondAccountV2 {
    /// Tag
    pub tag: Tag,

    /// Owner of the stake account
    pub owner: Pubkey,

    /// Amount locked in the account
    pub amount: u64,

    /// Pool to which the account belongs to
    pub pool: Pubkey,

    /// Offset of a last day where rewards were claimed from the contract creation date
    pub last_claimed_offset: u64,

    /// Minimum lockable amount of the pool when the account
    /// was created
    pub pool_minimum_at_creation: u64,

    // Unlock start date
    pub unlock_date: Option<i64>, // todo decide if Option None or -1 for uninitialized
}


#[allow(missing_docs)]
impl BondAccountV2 {
    pub const SEED: &'static [u8; 12] = b"bond_account_v2";

    pub fn create_key(
        owner: &Pubkey,
        stake_pool: &Pubkey,
        amount: u64,
        unlock_date: Option<i64>,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[
            BondAccountV2::SEED,
            &owner.to_bytes(),
            &stake_pool.to_bytes(),
            &amount.to_le_bytes(),
            &unlock_date.unwrap_or(0).to_le_bytes(),
        ];
        Pubkey::find_program_address(seeds, program_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner: Pubkey,
        pool: Pubkey,
        pool_minimum_at_creation: u64,
        amount: u64,
        unlock_date: Option<i64>,
    ) -> Self {
        Self {
            tag: Tag::BondAccountV2,
            owner,
            amount,
            pool,
            last_claimed_offset: 0,
            pool_minimum_at_creation,
            unlock_date,
        }
    }

    pub fn save(&self, mut dst: &mut [u8]) -> ProgramResult {
        self.serialize(&mut dst)
            .map_err(|_| ProgramError::InvalidAccountData)
    }

    pub fn from_account_info(
        a: &AccountInfo,
        allow_inactive: bool,
    ) -> Result<BondAccountV2, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        let tag = Tag::BondAccountV2;
        if data[0] != tag as u8 && data[0] != Tag::Uninitialized as u8 {
            return Err(AccessError::DataTypeMismatch.into());
        }
        let result = BondAccount::deserialize(&mut data)?;
        Ok(result)
    }
}