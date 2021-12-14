use bonfida_utils::InstructionsAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{instruction::Instruction, pubkey::Pubkey};

pub use crate::processor::{
    change_inflation, claim_pool_rewards, claim_rewards, close_stake_account, close_stake_pool,
    crank, create_central_state, create_stake_account, create_stake_pool, stake, unstake,
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

    /// Claim rewards for stake pool owner
    ///
    /// Accounts expected by this instruction:
    ///
    /// | Index | Writable | Signer | Description                   |
    /// |-------|----------|--------|-------------------------------|
    /// | 0     | ✅        | ❌      | The stake pool account        |
    /// | 1     | ✅        | ✅      | The owner of the stake pool   |
    /// | 2     | ✅        | ❌      | The rewards destination       |
    /// | 3     | ❌        | ❌      | The central state account     |
    /// | 4     | ❌        | ❌      | The mint account              |
    /// | 5     | ❌        | ❌      | The central vault account     |
    /// | 6     | ✅        | ❌      | The source rewards account    |
    /// | 7     | ❌        | ❌      | The SPL token program account |
    ClaimPoolRewards(claim_pool_rewards::Params),

    /// Claim rewards for staker
    ///
    /// Accounts expected by this instruction:
    ///
    /// | Index | Writable | Signer | Description                     |
    /// |-------|----------|--------|---------------------------------|
    /// | 0     | ✅        | ❌      | The stake pool account          |
    /// | 1     | ✅        | ❌      | The stake account               |
    /// | 2     | ✅        | ✅      | The owner of the stake account  |
    /// | 3     | ✅        | ❌      | The rewards destination account |
    /// | 4     | ❌        | ❌      | The central state account       |
    /// | 5     | ❌        | ❌      | The mint account                |
    /// | 6     | ❌        | ❌      | The central vault account       |
    /// | 7     | ✅        | ❌      | The source rewards account      |
    /// | 8     | ❌        | ❌      | The SPL token program account   |
    ClaimRewards(claim_rewards::Params),

    /// Permissionless crank to update the buffer of the stake pool
    ///
    /// Accounts expected by this instructions:
    ///
    /// | Index | Writable | Signer | Description               |
    /// |-------|----------|--------|---------------------------|
    /// | 0     | ✅        | ❌      | The stake pool account    |
    /// | 1     | ❌        | ❌      | The central state account |
    Crank(crank::Params),

    /// Close stake pool
    ///
    /// Accounts expected by this instruction
    ///
    // | Index | Writable | Signer | Description                |
    /// |-------|----------|--------|----------------------------|
    /// | 0     | ✅        | ❌      | The stake pool account     |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ✅        | ✅      | The stake pool owner       |
    CloseStakePool(close_stake_pool::Params),

    /// Close stake account
    ///
    /// Accounts expected by this instruction
    ///
    /// | Index | Writable | Signer | Description                |
    /// |-------|----------|--------|----------------------------|
    /// | 0     | ✅        | ❌      | The stake account          |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ✅        | ✅      | The stake pool owner       |
    CloseStakeAccount(close_stake_account::Params),

    /// Change inflation rate
    ///
    /// Accounts expected by this instruction
    ///
    /// | Index | Writable | Signer | Description                         |
    /// |-------|----------|--------|-------------------------------------|
    /// | 0     | ✅        | ❌      | The central state account           |
    /// | 1     | ❌        | ✅      | The central state authority account |
    ChangeInflation(change_inflation::Params),
}

pub fn create_central_state(
    accounts: create_central_state::Accounts<Pubkey>,
    params: create_central_state::Params,
) -> Instruction {
    accounts.get_instruction(MediaInstruction::CreateCentralState as u8, params)
}

pub fn create_stake_pool(
    accounts: create_stake_pool::Accounts<Pubkey>,
    params: create_stake_pool::Params,
) -> Instruction {
    accounts.get_instruction(MediaInstruction::CreateStakePool as u8, params)
}

pub fn create_stake_account(
    accounts: create_stake_account::Accounts<Pubkey>,
    params: create_stake_account::Params,
) -> Instruction {
    accounts.get_instruction(MediaInstruction::CreateStakeAccount as u8, params)
}

pub fn stake(accounts: stake::Accounts<Pubkey>, params: stake::Params) -> Instruction {
    accounts.get_instruction(MediaInstruction::Stake as u8, params)
}

pub fn unstake(accounts: unstake::Accounts<Pubkey>, params: unstake::Params) -> Instruction {
    accounts.get_instruction(MediaInstruction::Unstake as u8, params)
}

pub fn claim_pool_rewards(
    accounts: claim_pool_rewards::Accounts<Pubkey>,
    params: claim_pool_rewards::Params,
) -> Instruction {
    accounts.get_instruction(MediaInstruction::ClaimPoolRewards as u8, params)
}

pub fn claim_rewards(
    accounts: claim_rewards::Accounts<Pubkey>,
    params: claim_rewards::Params,
) -> Instruction {
    accounts.get_instruction(MediaInstruction::ClaimRewards as u8, params)
}

pub fn crank(accounts: crank::Accounts<Pubkey>, params: crank::Params) -> Instruction {
    accounts.get_instruction(MediaInstruction::Crank as u8, params)
}

pub fn close_stake_pool(
    accounts: close_stake_pool::Accounts<Pubkey>,
    params: close_stake_pool::Params,
) -> Instruction {
    accounts.get_instruction(MediaInstruction::CloseStakePool as u8, params)
}

pub fn close_stake_account(
    accounts: close_stake_account::Accounts<Pubkey>,
    params: close_stake_account::Params,
) -> Instruction {
    accounts.get_instruction(MediaInstruction::CloseStakeAccount as u8, params)
}

pub fn change_inflation(
    accounts: change_inflation::Accounts<Pubkey>,
    params: change_inflation::Params,
) -> Instruction {
    accounts.get_instruction(MediaInstruction::ChangeInflation as u8, params)
}
