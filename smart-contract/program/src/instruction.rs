pub use crate::processor::{
    change_inflation, claim_pool_rewards, claim_rewards, close_stake_account, close_stake_pool,
    crank, create_central_state, create_stake_account, create_stake_pool, stake, unstake,
};
use bonfida_utils::InstructionsAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{instruction::Instruction, pubkey::Pubkey};

#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
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
    CreateCentralState,

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
    CreateStakePool,

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
    CreateStakeAccount,

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
    Stake,

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
    Unstake,

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
    ClaimPoolRewards,

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
    ClaimRewards,

    /// Permissionless crank to update the buffer of the stake pool
    ///
    /// Accounts expected by this instructions:
    ///
    /// | Index | Writable | Signer | Description               |
    /// |-------|----------|--------|---------------------------|
    /// | 0     | ✅        | ❌      | The stake pool account    |
    /// | 1     | ❌        | ❌      | The central state account |
    Crank,

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
    /// | Index | Writable | Signer | Description                         |
    /// |-------|----------|--------|-------------------------------------|
    /// | 0     | ✅        | ❌      | The central state account           |
    /// | 1     | ❌        | ✅      | The central state authority account |
    ChangeInflation,
}

pub fn create_central_state(
    program_id: Pubkey,
    accounts: create_central_state::Accounts<Pubkey>,
    params: create_central_state::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        MediaInstruction::CreateCentralState as u8,
        params,
    )
}

pub fn create_stake_pool(
    program_id: Pubkey,
    accounts: create_stake_pool::Accounts<Pubkey>,
    params: create_stake_pool::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::CreateStakePool as u8, params)
}

pub fn create_stake_account(
    program_id: Pubkey,
    accounts: create_stake_account::Accounts<Pubkey>,
    params: create_stake_account::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        MediaInstruction::CreateStakeAccount as u8,
        params,
    )
}

pub fn stake(
    program_id: Pubkey,
    accounts: stake::Accounts<Pubkey>,
    params: stake::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::Stake as u8, params)
}

pub fn unstake(
    program_id: Pubkey,
    accounts: unstake::Accounts<Pubkey>,
    params: unstake::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::Unstake as u8, params)
}

pub fn claim_pool_rewards(
    program_id: Pubkey,
    accounts: claim_pool_rewards::Accounts<Pubkey>,
    params: claim_pool_rewards::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::ClaimPoolRewards as u8, params)
}

pub fn claim_rewards(
    program_id: Pubkey,
    accounts: claim_rewards::Accounts<Pubkey>,
    params: claim_rewards::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::ClaimRewards as u8, params)
}

pub fn crank(
    program_id: Pubkey,
    accounts: crank::Accounts<Pubkey>,
    params: crank::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::Crank as u8, params)
}

pub fn close_stake_pool(
    program_id: Pubkey,
    accounts: close_stake_pool::Accounts<Pubkey>,
    params: close_stake_pool::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::CloseStakePool as u8, params)
}

pub fn close_stake_account(
    program_id: Pubkey,
    accounts: close_stake_account::Accounts<Pubkey>,
    params: close_stake_account::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        MediaInstruction::CloseStakeAccount as u8,
        params,
    )
}

pub fn change_inflation(
    program_id: Pubkey,
    accounts: change_inflation::Accounts<Pubkey>,
    params: change_inflation::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::ChangeInflation as u8, params)
}
