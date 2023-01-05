pub use crate::processor::{
    activate_stake_pool, admin_freeze, admin_mint, change_central_state_authority,
    change_inflation, change_pool_minimum, change_pool_multiplier, claim_bond, claim_bond_rewards,
    claim_pool_rewards, claim_rewards, close_stake_account, close_stake_pool, crank, create_bond,
    create_central_state, create_stake_account, create_stake_pool, edit_metadata,
    sign_bond, stake, unlock_bond_tokens, unstake,
};
use bonfida_utils::InstructionsAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Create central state
    ///
    /// | Index | Writable | Signer | Description                  |
    /// | -------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account    |
    /// | 1     | ❌        | ❌      | The system program account   |
    /// | 2     | ✅        | ✅      | The fee payer account        |
    /// | 3     | ❌        | ❌      | The mint of the ACCESS token |
    /// | 4     | ✅        | ❌      | The metadata account         |
    /// | 5     | ❌        | ❌      | The metadata program account |
    /// | 6     | ❌        | ❌      | The rent sysvar account      |
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
    /// Activate a stake pool
    ///
    /// | Index | Writable | Signer | Description                      |
    /// | ------------------------------------------------------------ |
    /// | 0     | ❌        | ✅      | The central state authority      |
    /// | 1     | ✅        | ❌      | The stake pool to activate       |
    /// | 2     | ❌        | ❌      | The account of the central state |
    ActivateStakePool,
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
    /// | 0     | ✅        | ❌      | The central state account              |
    /// | 1     | ✅        | ❌      | The stake account                      |
    /// | 2     | ✅        | ❌      | The stake pool account                 |
    /// | 3     | ❌        | ✅      | The owner of the stake account         |
    /// | 4     | ✅        | ❌      | The source account of the stake tokens |
    /// | 5     | ❌        | ❌      | The SPL token program account          |
    /// | 6     | ✅        | ❌      | The stake pool vault account           |
    /// | 7     | ✅        | ❌      | The stake fee account                  |
    Stake,
    /// Unstake
    ///
    /// | Index | Writable | Signer | Description                    |
    /// | ---------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account      |
    /// | 1     | ✅        | ❌      | The stake account              |
    /// | 2     | ✅        | ❌      | The stake pool account         |
    /// | 3     | ❌        | ✅      | The owner of the stake account |
    // todo update the documentation - maybe not just here
    Unstake,
    /// Unused, but removing would cause the shift of all the other instructions indices
    // todo delete
    ExecuteUnstake,
    /// Claim rewards of a stake pool
    /// This instruction is used by stake pool owner for claiming their staking rewards
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
    /// This instruction can be used by stakers to claim their staking rewards
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
    /// This instructions updates the circular buffer with the pool balances multiplied by the current inflation
    ///
    /// | Index | Writable | Signer | Description                      |
    /// | ------------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The stake pool account           |
    /// | 1     | ❌        | ❌      | The account of the central state |
    Crank,
    /// Close a stake pool
    /// This instruction can be used to close an empty stake pool and collect the lamports
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The account of the stake pool |
    /// | 1     | ❌        | ❌      | Pool vault                    |
    /// | 2     | ✅        | ✅      | The owner of the stake pool   |
    CloseStakePool,
    /// Close a stake account
    /// This instruction can be used to close an empty stake account and collect the lamports
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
    /// This instruction can be used by authorized sellers to create a bond
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ✅        | ✅      | The bond seller account    |
    /// | 1     | ✅        | ❌      | The bond account           |
    /// | 2     | ❌        | ❌      |                            |
    /// | 3     | ❌        | ❌      | The system program account |
    /// | 4     | ✅        | ✅      | The fee account            |
    CreateBond,
    /// Sign a bond
    /// This instruction is used by authorized sellers to approve the creation of a bond
    ///
    /// | Index | Writable | Signer | Description |
    /// | --------------------------------------- |
    /// | 0     | ❌        | ✅      |             |
    /// | 1     | ✅        | ❌      |             |
    SignBond,
    /// Unlock ACCESS tokens bought through a bond account
    /// When tokens are unlocked they are withdrawn from the pool and are not considered staked anymore
    ///
    /// | Index | Writable | Signer | Description                      |
    /// | ------------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The bond account                 |
    /// | 1     | ❌        | ✅      | The account of the bond owner    |
    /// | 2     | ❌        | ❌      | The ACCESS mint token            |
    /// | 3     | ✅        | ❌      | The ACCESS token destination     |
    /// | 4     | ✅        | ❌      | The account of the central state |
    /// | 5     | ✅        | ❌      | The account of the staking pool  |
    /// | 6     | ✅        | ❌      | The vault of the staking pool    |
    /// | 7     | ❌        | ❌      | The SPL token program account    |
    UnlockBondTokens,
    /// Claim bond
    /// This instruction allows a buyer to claim a bond once it has been signed by enough DAO members.
    ///
    /// | Index | Writable | Signer | Description                                      |
    /// | ---------------------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The bond account                                 |
    /// | 1     | ❌        | ✅      | The account of the bond buyer                    |
    /// | 2     | ✅        | ❌      | The token account used to purchase the bond      |
    /// | 3     | ✅        | ❌      | The token account where the sell proceed is sent |
    /// | 4     | ✅        | ❌      | The stake pool account                           |
    /// | 5     | ✅        | ❌      | The mint of the ACCESS token                     |
    /// | 6     | ✅        | ❌      | The vault of the stake pool                      |
    /// | 7     | ✅        | ❌      | The central state account                        |
    /// | 8     | ❌        | ❌      | The SPL token program account                    |
    ClaimBond,
    /// Claim bond rewards
    /// This Instruction allows bond owners to claim their staking rewards
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
    /// This instruction allows a pool owner to adjust the price of its subscription for new joiners without impacting people who already subscribed
    ///
    /// | Index | Writable | Signer | Description                  |
    /// | -------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account       |
    /// | 1     | ❌        | ✅      | The stake pool owner account |
    ChangePoolMinimum,
    /// Allows central state authority to mint ACCESS tokens
    ///
    /// | Index | Writable | Signer | Description                      |
    /// | ------------------------------------------------------------ |
    /// | 0     | ❌        | ✅      | The central state authority      |
    /// | 1     | ✅        | ❌      | The ACCESS mint token            |
    /// | 2     | ✅        | ❌      | The ACCESS token destination     |
    /// | 3     | ❌        | ❌      | The account of the central state |
    /// | 4     | ❌        | ❌      | The SPL token program account    |
    AdminMint,
    /// Freeze and unfreeze a program account
    /// This admin instruction can be dangereous 💀
    ///
    /// | Index | Writable | Signer | Description                         |
    /// | --------------------------------------------------------------- |
    /// | 0     | ❌        | ✅      | The central state authority         |
    /// | 1     | ✅        | ❌      | The account to freeze (or unfreeze) |
    /// | 2     | ❌        | ❌      | The account of the central state    |
    AdminFreeze,
    ///
    /// | Index | Writable | Signer | Description                  |
    /// | -------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account       |
    /// | 1     | ❌        | ✅      | The stake pool owner account |
    ChangePoolMultiplier,
    /// Change central state authority
    ///
    /// | Index | Writable | Signer | Description                                |
    /// | ---------------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The account of the central state           |
    /// | 1     | ❌        | ✅      | The account of the central state authority |
    ChangeCentralStateAuthority,
    /// Edit metadata
    ///
    /// | Index | Writable | Signer | Description                                |
    /// | ---------------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The central state account                  |
    /// | 1     | ❌        | ✅      | The account of the central state authority |
    /// | 2     | ✅        | ❌      | The metadata account                       |
    /// | 3     | ❌        | ❌      | The metadata program account               |
    EditMetadata,
}
#[allow(missing_docs)]
pub fn create_central_state(
    program_id: Pubkey,
    accounts: create_central_state::Accounts<Pubkey>,
    params: create_central_state::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::CreateCentralState as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn create_stake_pool(
    program_id: Pubkey,
    accounts: create_stake_pool::Accounts<Pubkey>,
    params: create_stake_pool::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::CreateStakePool as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn create_stake_account(
    program_id: Pubkey,
    accounts: create_stake_account::Accounts<Pubkey>,
    params: create_stake_account::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::CreateStakeAccount as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn stake(
    program_id: Pubkey,
    accounts: stake::Accounts<Pubkey>,
    params: stake::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::Stake as u8, params)
}
#[allow(missing_docs)]
pub fn unstake(
    program_id: Pubkey,
    accounts: unstake::Accounts<Pubkey>,
    params: unstake::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::Unstake as u8, params)
}
#[allow(missing_docs)]
pub fn claim_pool_rewards(
    program_id: Pubkey,
    accounts: claim_pool_rewards::Accounts<Pubkey>,
    params: claim_pool_rewards::Params,
    owner_must_sign: bool,
) -> Instruction {
    let mut ix = accounts.get_instruction(
        program_id,
        ProgramInstruction::ClaimPoolRewards as u8,
        params,
    );

    if let Some(acc) = ix.accounts.get_mut(1) {
        acc.is_signer = owner_must_sign
    }

    ix
}
#[allow(missing_docs)]
pub fn claim_rewards(
    program_id: Pubkey,
    accounts: claim_rewards::Accounts<Pubkey>,
    params: claim_rewards::Params,
    owner_must_sign: bool,
) -> Instruction {
    let mut ix =
        accounts.get_instruction(program_id, ProgramInstruction::ClaimRewards as u8, params);

    if let Some(acc) = ix.accounts.get_mut(2) {
        acc.is_signer = owner_must_sign
    }

    ix
}
#[allow(missing_docs)]
pub fn crank(
    program_id: Pubkey,
    accounts: crank::Accounts<Pubkey>,
    params: crank::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::Crank as u8, params)
}
#[allow(missing_docs)]
pub fn close_stake_pool(
    program_id: Pubkey,
    accounts: close_stake_pool::Accounts<Pubkey>,
    params: close_stake_pool::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::CloseStakePool as u8, params)
}
#[allow(missing_docs)]
pub fn close_stake_account(
    program_id: Pubkey,
    accounts: close_stake_account::Accounts<Pubkey>,
    params: close_stake_account::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::CloseStakeAccount as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn change_inflation(
    program_id: Pubkey,
    accounts: change_inflation::Accounts<Pubkey>,
    params: change_inflation::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::ChangeInflation as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn create_bond(
    program_id: Pubkey,
    accounts: create_bond::Accounts<Pubkey>,
    params: create_bond::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::CreateBond as u8, params)
}
#[allow(missing_docs)]
pub fn sign_bond(
    program_id: Pubkey,
    accounts: sign_bond::Accounts<Pubkey>,
    params: sign_bond::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::SignBond as u8, params)
}
#[allow(missing_docs)]
pub fn unlock_bond_tokens(
    program_id: Pubkey,
    accounts: unlock_bond_tokens::Accounts<Pubkey>,
    params: unlock_bond_tokens::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::UnlockBondTokens as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn claim_bond(
    program_id: Pubkey,
    accounts: claim_bond::Accounts<Pubkey>,
    params: claim_bond::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::ClaimBond as u8, params)
}
#[allow(missing_docs)]
pub fn claim_bond_rewards(
    program_id: Pubkey,
    accounts: claim_bond_rewards::Accounts<Pubkey>,
    params: claim_bond_rewards::Params,
    owner_must_sign: bool,
) -> Instruction {
    let mut ix = accounts.get_instruction(
        program_id,
        ProgramInstruction::ClaimBondRewards as u8,
        params,
    );

    if let Some(acc) = ix.accounts.get_mut(2) {
        acc.is_signer = owner_must_sign
    }

    ix
}
#[allow(missing_docs)]
pub fn change_pool_minimum(
    program_id: Pubkey,
    accounts: change_pool_minimum::Accounts<Pubkey>,
    params: change_pool_minimum::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::ChangePoolMinimum as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn admin_mint(
    program_id: Pubkey,
    accounts: admin_mint::Accounts<Pubkey>,
    params: admin_mint::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::AdminMint as u8, params)
}
#[allow(missing_docs)]
pub fn admin_freeze(
    program_id: Pubkey,
    accounts: admin_freeze::Accounts<Pubkey>,
    params: admin_freeze::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::AdminFreeze as u8, params)
}
#[allow(missing_docs)]
pub fn activate_stake_pool(
    program_id: Pubkey,
    accounts: activate_stake_pool::Accounts<Pubkey>,
    params: activate_stake_pool::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::ActivateStakePool as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn change_pool_multiplier(
    program_id: Pubkey,
    accounts: change_pool_multiplier::Accounts<Pubkey>,
    params: change_pool_multiplier::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::ChangePoolMultiplier as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn change_central_state_authority(
    program_id: Pubkey,
    accounts: change_central_state_authority::Accounts<Pubkey>,
    params: change_central_state_authority::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::ChangeCentralStateAuthority as u8,
        params,
    )
}
#[allow(missing_docs)]
pub fn edit_metadata(
    program_id: Pubkey,
    accounts: edit_metadata::Accounts<Pubkey>,
    params: edit_metadata::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::EditMetadata as u8, params)
}
