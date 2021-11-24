use borsh::{BorshDeserialize, BorshSerialize};

pub use crate::processor::{
    change_inflation, create_central_state, create_stake_account, create_stake_pool, stake, unstake,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum MediaInstruction {
    // Create central state
    CreateCentralState(create_central_state::Params),

    // Create stake pool
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
    ChangeInflationRate(change_inflation::Params),
}
