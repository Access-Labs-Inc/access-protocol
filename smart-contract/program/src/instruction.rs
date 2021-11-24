use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};

pub use crate::processor::{
    change_inflation, create_central_state, create_stake_account, create_stake_pool, stake, unstake,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum MediaInstruction {
    /// Create central state
    ///
    /// Accounts expected by this instruction:
    ///
    /// 1. `[writable]` The central state account
    /// 2. `[]` The system program account
    /// 3. `[writable, signer]` The fee payer account
    /// 4. `[]` The rent sysvar account
    CreateCentralState(create_central_state::Params),

    /// Create stake pool
    ///
    /// Accounts expected by this instruction:
    ///
    /// 1. `[writable]` The stake pool account
    /// 2. `[]` The system program account
    /// 3. `[writable, signer]` The fee payer account
    /// 4. `[]` The rent sysvar account
    CreateStakePool(create_stake_pool::Params),

    // Create stake account
    CreateStakeAccount(create_stake_account::Params),

    // Stake tokens into a stake pool
    Stake(stake::Params),

    // Unstake tokens from a stake pool
    Unstake(unstake::Params),

    // Permissionless crank used to transfer tokens
    // to stake pools owners
    ClaimRewards,

    // Close stake pool
    CloseStakePool,

    // Close stake account
    CloseStakeAccount,

    // Change inflation rate
    ChangeInflation(change_inflation::Params),
}
