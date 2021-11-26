use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};

pub use crate::processor::{
    change_inflation, close_stake_account, close_stake_pool, create_central_state,
    create_stake_account, create_stake_pool, stake, unstake,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum MediaInstruction {
    /// Create central state
    ///
    /// Accounts expected by this instruction:
    ///
    // | Index | Writable | Signer | Description                |
    // |-------|----------|--------|----------------------------|
    // | 0     | ✅        | ❌      | The central state account  |
    // | 1     | ❌        | ❌      | The system program account |
    // | 2     | ✅        | ✅      | The fee payer account      |
    // | 3     | ❌        | ❌      | The rent sysvar account    |
    CreateCentralState(create_central_state::Params),

    /// Create stake pool
    ///
    /// Accounts expected by this instruction:
    ///
    /// | Index | Writable | Signer | Description                |
    /// |-------|----------|--------|----------------------------|
    /// | 0     | ✅        | ❌      | The stake pool account     |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ✅        | ✅      | The fee payer account      |
    /// | 3     | ❌        | ❌      | The rent sysvar account    |
    CreateStakePool(create_stake_pool::Params),

    /// Create stake account
    ///
    /// Accounts expected by this instruction:
    ///
    /// | Index | Writable | Signer | Description                |
    /// |-------|----------|--------|----------------------------|
    /// | 0     | ✅        | ❌      | The stake account          |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ✅        | ✅      | The fee payer account      |
    /// | 3     | ❌        | ❌      | The rent sysvar account    |
    CreateStakeAccount(create_stake_account::Params),

    /// Stake tokens into a stake pool
    ///
    /// Accounts expected by this instruction:
    ///
    /// | Index | Writable | Signer | Description                   |
    /// |-------|----------|--------|-------------------------------|
    /// | 0     | ✅        | ❌      | The stake account             |
    /// | 1     | ✅        | ❌      | The stake pool account        |
    /// | 2     | ✅        | ✅      | The stake account owner       |
    /// | 3     | ✅        | ❌      | The source token account      |
    /// | 4     | ❌        | ❌      | The SPL token program account |
    /// | 5     | ✅        | ❌      | The vault token account       |
    Stake(stake::Params),

    /// Unstake tokens from a stake pool
    ///
    /// Accounts expected by this instruction:
    ///
    /// | Index | Writable | Signer | Description                   |
    /// |-------|----------|--------|-------------------------------|
    /// | 0     | ✅        | ❌      | The stake account             |
    /// | 1     | ✅        | ❌      | The stake pool account        |
    /// | 2     | ✅        | ✅      | The stake account owner       |
    /// | 3     | ✅        | ❌      | The destination token account |
    /// | 4     | ❌        | ❌      | The SPL token program account |
    /// | 5     | ✅        | ❌      | The vault token account       |
    Unstake(unstake::Params),

    /// Permissionless crank used to transfer tokens to stake pools owners
    ///
    /// Accounts expected by this instruction:
    ///
    /// TODO
    ClaimRewards,

    /// Close stake pool
    ///
    /// Accounts expected by this instruction
    ///
    // | Index | Writable | Signer | Description                |
    /// |-------|----------|--------|----------------------------|
    /// | 0     | ✅        | ❌      | The stake pool account     |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ✅        | ✅      | The stake pool owner       |
    CloseStakePool,

    /// Close stake account
    ///
    /// Accounts expected by this instruction
    ///
    /// | Index | Writable | Signer | Description                |
    /// |-------|----------|--------|----------------------------|
    /// | 0     | ✅        | ❌      | The stake account          |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ✅        | ✅      | The stake pool owner       |
    CloseStakeAccount,

    /// Change inflation rate
    ///
    /// Accounts expected by this instruction
    ///
    /// TODO
    ChangeInflation(change_inflation::Params),
}
