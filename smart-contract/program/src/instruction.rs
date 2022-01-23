pub use crate::processor::{
    change_inflation, change_pool_minimum, claim_bond, claim_bond_rewards, claim_pool_rewards,
    claim_rewards, close_stake_account, close_stake_pool, crank, create_bond, create_central_state,
    create_stake_account, create_stake_pool, sign_bond, stake, unlock_bond_tokens, unstake,
};
use bonfida_utils::InstructionsAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum MediaInstruction {
    /// Create central state
    ///
    /// | Index | Writable | Signer | Description                  |
    /// | -------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake account            |
    /// | 1     | ❌        | ❌      | The system program account   |
    /// | 2     | ✅        | ✅      | The fee payer account        |
    /// | 3     | ❌        | ❌      | The mint of the ACCESS token |
    CreateCentralState,
    /// Create stake pool
    ///
    /// | Index | Writable | Signer | Description                  |
    /// | -------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account       |
    /// | 1     | ❌        | ❌      | The system program account   |
    /// | 2     | ✅        | ✅      | The fee payer account        |
    /// | 3     | ❌        | ❌      | The stake pool vault account |
    CreateStakePool,
    /// Create stake account
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The stake account          |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ❌        | ❌      | The stake pool account     |
    /// | 3     | ✅        | ✅      | The fee payer account      |
    CreateStakeAccount,
    /// Stake
    ///
    /// | Index | Writable | Signer | Description                            |
    /// | ------------------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The stake account                      |
    /// | 1     | ✅        | ❌      | The stake pool account                 |
    /// | 2     | ❌        | ✅      | The owner of the stake account         |
    /// | 3     | ✅        | ❌      | The source account of the stake tokens |
    /// | 4     | ❌        | ❌      | The SPL token program account          |
    /// | 5     | ✅        | ❌      | The stake pool vault account           |
    Stake,
    /// Unstake
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake account                    |
    /// | 1     | ✅        | ❌      | The stake pool account               |
    /// | 2     | ❌        | ✅      | The owner of the stake account       |
    /// | 3     | ✅        | ❌      | The destination of the staked tokens |
    /// | 4     | ❌        | ❌      | The SPL token program account        |
    /// | 5     | ✅        | ❌      | The stake pool vault                 |
    Unstake,
    /// Claim rewards of a stake pool
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account               |
    /// | 1     | ❌        | ✅      | The stake pool owner account         |
    /// | 2     | ✅        | ❌      | The rewards destination              |
    /// | 3     | ❌        | ❌      | The central state account            |
    /// | 4     | ✅        | ❌      | The mint address of the ACCESS token |
    /// | 5     | ❌        | ❌      | The SPL token program account        |
    ClaimPoolRewards,
    /// Claim rewards of a stake account
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account               |
    /// | 1     | ✅        | ❌      | The stake account                    |
    /// | 2     | ❌        | ✅      | The owner of the stake account       |
    /// | 3     | ✅        | ❌      | The rewards destination              |
    /// | 4     | ❌        | ❌      | The central state account            |
    /// | 5     | ✅        | ❌      | The mint address of the ACCESS token |
    /// | 6     | ❌        | ❌      | The SPL token program account        |
    ClaimRewards,
    /// Permissionless crank to update the stake pool rewards
    ///
    /// | Index | Writable | Signer | Description                      |
    /// | ------------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The stake pool account           |
    /// | 1     | ❌        | ❌      | The account of the central state |
    Crank,
    /// Close a stake pool
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The account of the stake pool |
    /// | 1     | ✅        | ✅      | The owner of the stake pool   |
    CloseStakePool,
    /// Close a stake account
    ///
    /// | Index | Writable | Signer | Description                    |
    /// | ---------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake account              |
    /// | 1     | ✅        | ✅      | The owner of the stake account |
    CloseStakeAccount,
    /// Change central state inflation
    ///
    /// | Index | Writable | Signer | Description                                |
    /// | ---------------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The account of the central state           |
    /// | 1     | ❌        | ✅      | The account of the central state authority |
    ChangeInflation,
    /// Create a bond
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ✅        | ✅      | The bond seller account    |
    /// | 1     | ✅        | ❌      | The bond account           |
    /// | 2     | ❌        | ❌      |                            |
    /// | 3     | ❌        | ❌      | The system program account |
    /// | 4     | ❌        | ❌      | The fee account            |
    CreateBond,
    /// Sign a bond
    ///
    /// | Index | Writable | Signer | Description |
    /// | --------------------------------------- |
    /// | 0     | ❌        | ✅      |             |
    /// | 1     | ✅        | ❌      |             |
    SignBond,
    /// Unlock ACCESS tokens bought through a bond account
    ///
    /// | Index | Writable | Signer | Description                      |
    /// | ------------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The bond account                 |
    /// | 1     | ❌        | ✅      | The account of the bond owner    |
    /// | 2     | ✅        | ❌      | The ACCESS mint token            |
    /// | 3     | ✅        | ❌      | The ACCESS token destination     |
    /// | 4     | ❌        | ❌      | The account of the central state |
    /// | 5     | ❌        | ❌      | The SPL token program account    |
    UnlockBondTokens,
    /// Claim a bond after it has been issued and signed
    ///
    /// | Index | Writable | Signer | Description                                      |
    /// | ---------------------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The bond account                                 |
    /// | 1     | ❌        | ✅      | The account of the bond buyer                    |
    /// | 2     | ✅        | ❌      | The token account used to purchase the bond      |
    /// | 3     | ✅        | ❌      | The token account where the sell proceed is sent |
    /// | 4     | ❌        | ❌      | The SPL token program account                    |
    ClaimBond,
    /// Claim bond rewards
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account               |
    /// | 1     | ✅        | ❌      | The bond account                     |
    /// | 2     | ❌        | ✅      | The bond account owner               |
    /// | 3     | ✅        | ❌      | The rewards destination              |
    /// | 4     | ❌        | ❌      | The central state account            |
    /// | 5     | ✅        | ❌      | The mint address of the ACCESS token |
    /// | 6     | ❌        | ❌      | The SPL token program account        |
    ClaimBondRewards,
    /// Change the minimum stakeable amount of a pool
    ///
    /// | Index | Writable | Signer | Description            |
    /// | -------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account |
    /// | 1     | ❌        | ✅      | The bond account       |
    ChangePoolMinimum,
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
pub fn create_bond(
    program_id: Pubkey,
    accounts: create_bond::Accounts<Pubkey>,
    params: create_bond::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::CreateBond as u8, params)
}
pub fn sign_bond(
    program_id: Pubkey,
    accounts: sign_bond::Accounts<Pubkey>,
    params: sign_bond::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::SignBond as u8, params)
}
pub fn unlock_bond_tokens(
    program_id: Pubkey,
    accounts: unlock_bond_tokens::Accounts<Pubkey>,
    params: unlock_bond_tokens::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::UnlockBondTokens as u8, params)
}
pub fn claim_bond(
    program_id: Pubkey,
    accounts: claim_bond::Accounts<Pubkey>,
    params: claim_bond::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::ClaimBond as u8, params)
}
pub fn claim_bond_rewards(
    program_id: Pubkey,
    accounts: claim_bond_rewards::Accounts<Pubkey>,
    params: claim_bond_rewards::Params,
) -> Instruction {
    accounts.get_instruction(program_id, MediaInstruction::ClaimBondRewards as u8, params)
}
pub fn change_pool_minimum(
    program_id: Pubkey,
    accounts: change_pool_minimum::Accounts<Pubkey>,
    params: change_pool_minimum::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        MediaInstruction::ChangePoolMinimum as u8,
        params,
    )
}
