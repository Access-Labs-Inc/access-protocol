use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{instruction::Instruction, pubkey::Pubkey};

pub use crate::processor::{
    activate_stake_pool, add_to_bond_v2, admin_change_freeze_authority, admin_freeze, admin_mint,
    admin_program_freeze, admin_renounce, admin_set_protocol_fee, admin_setup_fee_split,
    change_central_state_authority, change_inflation, change_pool_minimum, change_pool_multiplier, claim_bond,
    claim_bond_rewards, claim_bond_v2_rewards, claim_pool_rewards, claim_rewards,
    close_stake_account, close_stake_pool, crank, create_bond, create_bond_v2,
    create_central_state, create_stake_account, create_stake_pool, distribute_fees,
    edit_metadata, migrate_central_state_v2, sign_bond, stake, unlock_bond_tokens, unlock_bond_v2,
    unstake, create_royalty_account, close_royalty_account, cpi_claim_rewards, cpi_claim_bond_v2_rewards
};

// todo regenerate
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, BorshSize, FromPrimitive, Copy, Clone)]
pub enum ProgramInstruction {
    /// Create central state
    ///
    /// | Index | Writable | Signer | Description                  |
    /// | -------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account    |
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
    /// | 4     | ❌        | ❌      | The central state account    |
    CreateStakePool,
    /// Activate a stake pool
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The stake pool to activate |
    /// | 1     | ❌        | ❌      | The central state account  |
    ActivateStakePool,
    /// Create stake account
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The stake account          |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ❌        | ❌      | The stake pool account     |
    /// | 3     | ✅        | ✅      | The fee payer account      |
    /// | 4     | ❌        | ❌      | The central state account  |
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
    /// | 7     | ✅        | ❌      | The central state ATA                  |
    Stake,
    /// Unstake
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account            |
    /// | 1     | ✅        | ❌      | The stake account                    |
    /// | 2     | ✅        | ❌      | The stake pool account               |
    /// | 3     | ❌        | ✅      | The owner of the stake account       |
    /// | 4     | ✅        | ❌      | The destination of the staked tokens |
    /// | 5     | ❌        | ❌      | The SPL token program account        |
    /// | 6     | ✅        | ❌      | The stake pool vault                 |
    Unstake,
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
    /// | Index | Writable | Signer | Description                       |
    /// | ------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account            |
    /// | 1     | ✅        | ❌      | The stake account                 |
    /// | 2     | ❌        | ✅      | The owner of the stake account    |
    /// | 3     | ✅        | ❌      | The rewards destination           |
    /// | 4     | ❌        | ❌      | The central state account         |
    /// | 5     | ✅        | ❌      | The mint address of the ACS token |
    /// | 6     | ❌        | ❌      | The SPL token program account     |
    ClaimRewards,
    /// Permissionless crank to update the stake pool rewards
    /// This instructions updates the circular buffer with the pool balances multiplied by the current inflation
    ///
    /// | Index | Writable | Signer | Description               |
    /// | ----------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account    |
    /// | 1     | ✅        | ❌      | The central state account |
    Crank,
    /// Close a stake pool
    /// This instruction can be used to close an empty stake pool and collect the lamports
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The account of the stake pool |
    /// | 1     | ❌        | ❌      | Pool vault                    |
    /// | 2     | ✅        | ✅      | The owner of the stake pool   |
    /// | 3     | ❌        | ❌      | The central state account     |
    CloseStakePool,
    /// Close a stake account
    /// This instruction can be used to close an empty stake account and collect the lamports
    ///
    /// | Index | Writable | Signer | Description                    |
    /// | ---------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake account              |
    /// | 1     | ✅        | ✅      | The owner of the stake account |
    /// | 2     | ❌        | ❌      | The central state account      |
    CloseStakeAccount,
    /// Change central state inflation
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account            |
    /// | 1     | ❌        | ✅      | The central state account authority  |
    /// | 2     | ❌        | ❌      | The mint address of the ACCESS token |
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
    /// | 5     | ❌        | ❌      | The central state account  |
    CreateBond,
    /// Sign a bond
    /// This instruction is used by authorized sellers to approve the creation of a bond
    ///
    /// | Index | Writable | Signer | Description |
    /// | --------------------------------------- |
    /// | 0     | ❌        | ✅      |             |
    /// | 1     | ✅        | ❌      |             |
    /// | 2     | ❌        | ❌      |             |
    SignBond,
    /// Unlock ACCESS tokens bought through a bond account
    /// When tokens are unlocked they are withdrawn from the pool and are not considered staked anymore
    ///
    /// | Index | Writable | Signer | Description                     |
    /// | ----------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The bond account                |
    /// | 1     | ❌        | ✅      | The account of the bond owner   |
    /// | 2     | ❌        | ❌      | The ACCESS mint token           |
    /// | 3     | ✅        | ❌      | The ACCESS token destination    |
    /// | 4     | ✅        | ❌      | The central state account       |
    /// | 5     | ✅        | ❌      | The account of the staking pool |
    /// | 6     | ✅        | ❌      | The vault of the staking pool   |
    /// | 7     | ❌        | ❌      | The SPL token program account   |
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
    /// | 2     | ❌        | ❌      | The central state account    |
    ChangePoolMinimum,
    /// Allows central state authority to mint ACCESS tokens
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ❌        | ✅      | The central state authority   |
    /// | 1     | ✅        | ❌      | The ACCESS mint token         |
    /// | 2     | ✅        | ❌      | The ACCESS token destination  |
    /// | 3     | ❌        | ❌      | The central state account     |
    /// | 4     | ❌        | ❌      | The SPL token program account |
    AdminMint,
    /// Freeze and unfreeze a program account
    /// This admin instruction can be dangereous 💀
    ///
    /// | Index | Writable | Signer | Description                         |
    /// | --------------------------------------------------------------- |
    /// | 0     | ❌        | ✅      | The central state authority         |
    /// | 1     | ✅        | ❌      | The account to freeze (or unfreeze) |
    /// | 2     | ❌        | ❌      | The central state account           |
    AdminFreeze,
    ///
    /// | Index | Writable | Signer | Description                  |
    /// | -------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account       |
    /// | 1     | ❌        | ✅      | The stake pool owner account |
    /// | 2     | ❌        | ❌      | The central state account    |
    ChangePoolMultiplier,
    /// Change central state authority
    ///
    /// | Index | Writable | Signer | Description                         |
    /// | --------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account           |
    /// | 1     | ❌        | ✅      | The central state account authority |
    ChangeCentralStateAuthority,
    /// Edit metadata
    ///
    /// | Index | Writable | Signer | Description                         |
    /// | --------------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The central state account           |
    /// | 1     | ❌        | ✅      | The central state account authority |
    /// | 2     | ✅        | ❌      | The metadata account                |
    /// | 3     | ❌        | ❌      | The metadata program account        |
    EditMetadata,
    /// Create a Bond V2
    ///
    /// | Index | Writable | Signer | Description                       |
    /// | ------------------------------------------------------------- |
    /// | 0     | ✅        | ✅      | The fee account                   |
    /// | 1     | ✅        | ✅      | The bond seller account           |
    /// | 2     | ✅        | ❌      | From ATA                          |
    /// | 3     | ❌        | ❌      | The bond recipient wallet         |
    /// | 4     | ✅        | ❌      | The bond account                  |
    /// | 5     | ✅        | ❌      | Central state                     |
    /// | 6     | ✅        | ❌      | The vault of the central state    |
    /// | 7     | ✅        | ❌      | The pool account                  |
    /// | 8     | ✅        | ❌      | The vault of the pool             |
    /// | 9     | ✅        | ❌      | The mint address of the ACS token |
    /// | 10    | ❌        | ❌      | The SPL token program account     |
    /// | 11    | ❌        | ❌      | The system program account        |
    CreateBondV2,
    /// Add more tokens to an existing Bond V2
    ///
    /// | Index | Writable | Signer | Description                       |
    /// | ------------------------------------------------------------- |
    /// | 0     | ✅        | ✅      | The bond seller account           |
    /// | 1     | ✅        | ❌      | From ATA                          |
    /// | 2     | ✅        | ❌      | The bond account                  |
    /// | 3     | ✅        | ❌      | Central state                     |
    /// | 4     | ✅        | ❌      | The vault of the central state    |
    /// | 5     | ✅        | ❌      | The pool account                  |
    /// | 6     | ✅        | ❌      | The vault of the pool             |
    /// | 7     | ✅        | ❌      | The mint address of the ACS token |
    /// | 8     | ❌        | ❌      | The SPL token program account     |
    /// | 9     | ❌        | ❌      | The system program account        |
    AddToBondV2,
    /// Claim rewards of a bond V2
    ///
    /// | Index | Writable | Signer | Description                       |
    /// | ------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The stake pool account            |
    /// | 1     | ✅        | ❌      | The Bond V2 account               |
    /// | 2     | ❌        | ✅      | The owner of the Bond V2 account  |
    /// | 3     | ✅        | ❌      | The rewards destination           |
    /// | 4     | ❌        | ❌      | The central state account         |
    /// | 5     | ✅        | ❌      | The mint address of the ACS token |
    /// | 6     | ❌        | ❌      | The SPL token program account     |
    ClaimBondV2Rewards,
    /// Unlock bond v2
    ///
    /// | Index | Writable | Signer | Description                          |
    /// | ---------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account            |
    /// | 1     | ✅        | ❌      | The bond account                     |
    /// | 2     | ❌        | ✅      | The owner of the bond V2 account     |
    /// | 3     | ✅        | ❌      | The destination of the locked tokens |
    /// | 4     | ✅        | ❌      | The pool account                     |
    /// | 5     | ✅        | ❌      | The pool vault                       |
    /// | 6     | ❌        | ❌      | The SPL token program account        |
    UnlockBondV2,
    /// Setup fee split
    ///
    /// | Index | Writable | Signer | Description                 |
    /// | ------------------------------------------------------- |
    /// | 0     | ❌        | ✅      | The central state authority |
    /// | 1     | ✅        | ❌      | The central state account   |
    /// | 2     | ❌        | ❌      | The system program account  |
    AdminSetupFeeSplit,
    /// Distribute fees to the recipients
    ///
    /// | Index    | Writable | Signer | Description                                  |
    /// | --------------------------------------------------------------------------- |
    /// | 0        | ✅        | ❌      | The central state account                    |
    /// | 1        | ✅        | ❌      | The central state ATA                        |
    /// | 2        | ❌        | ❌      | The SPL token program account                |
    /// | 3        | ✅        | ❌      | The mint address of the ACS token            |
    /// | 4..4 + N | ✅        | ❌      | The token accounts to distribute the fees to |
    DistributeFees,
    /// Admin set protocol fee
    ///
    /// | Index | Writable | Signer | Description                 |
    /// | ------------------------------------------------------- |
    /// | 0     | ❌        | ✅      | The central state authority |
    /// | 1     | ✅        | ❌      | The central state account   |
    AdminSetProtocolFee,
    /// Migrate the central state to the v2 format
    ///
    /// | Index | Writable | Signer | Description                |
    /// | ------------------------------------------------------ |
    /// | 0     | ✅        | ❌      | The central state account  |
    /// | 1     | ❌        | ❌      | The system program account |
    /// | 2     | ✅        | ✅      | The fee payer account      |
    MigrateCentralStateV2,
    /// Admin program freeze instruction.
    ///
    /// | Index | Writable | Signer | Description                         |
    /// | --------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account           |
    /// | 1     | ❌        | ✅      | The central state account authority |
    AdminProgramFreeze,
    /// Admin renounce functionality
    ///
    /// | Index | Writable | Signer | Description                         |
    /// | --------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account           |
    /// | 1     | ❌        | ✅      | The central state account authority |
    AdminRenounce,
    /// Change freeze authority
    ///
    /// | Index | Writable | Signer | Description                         |
    /// | --------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The central state account           |
    /// | 1     | ❌        | ✅      | The central state account authority |
    AdminChangeFreezeAuthority,
    // todo doc
    CreateRoyaltyAccount,
    CloseRoyaltyAccount,
    CpiClaimRewards,
CpiClaimBondV2Rewards,
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
pub fn create_bond_v2(
    program_id: Pubkey,
    accounts: create_bond_v2::Accounts<Pubkey>,
    params: create_bond_v2::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::CreateBondV2 as u8, params)
}

#[allow(missing_docs)]
pub fn add_to_bond_v2(
    program_id: Pubkey,
    accounts: add_to_bond_v2::Accounts<Pubkey>,
    params: add_to_bond_v2::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::AddToBondV2 as u8, params)
}

#[allow(missing_docs)]
pub fn claim_bond_v2_rewards(
    program_id: Pubkey,
    accounts: claim_bond_v2_rewards::Accounts<Pubkey>,
    params: claim_bond_v2_rewards::Params,
    owner_must_sign: bool,
) -> Instruction {
    let mut ix = accounts.get_instruction(
        program_id,
        ProgramInstruction::ClaimBondV2Rewards as u8,
        params,
    );
    if let Some(acc) = ix.accounts.get_mut(2) {
        acc.is_signer = owner_must_sign
    }
    ix
}

#[allow(missing_docs)]
pub fn unlock_bond_v2(
    program_id: Pubkey,
    accounts: unlock_bond_v2::Accounts<Pubkey>,
    params: unlock_bond_v2::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::UnlockBondV2 as u8, params)
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

#[allow(missing_docs)]
pub fn admin_setup_fee_split(
    program_id: Pubkey,
    accounts: admin_setup_fee_split::Accounts<Pubkey>,
    params: admin_setup_fee_split::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::AdminSetupFeeSplit as u8,
        params,
    )
}

#[allow(missing_docs)]
pub fn distribute_fees(
    program_id: Pubkey,
    accounts: distribute_fees::Accounts<Pubkey>,
    params: distribute_fees::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::DistributeFees as u8, params)
}

#[allow(missing_docs)]
pub fn admin_set_protocol_fee(
    program_id: Pubkey,
    accounts: admin_set_protocol_fee::Accounts<Pubkey>,
    params: admin_set_protocol_fee::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::AdminSetProtocolFee as u8,
        params,
    )
}

#[allow(missing_docs)]
pub fn migrate_central_state_v2(
    program_id: Pubkey,
    accounts: migrate_central_state_v2::Accounts<Pubkey>,
    params: migrate_central_state_v2::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::MigrateCentralStateV2 as u8,
        params,
    )
}

#[allow(missing_docs)]
pub fn admin_program_freeze(
    program_id: Pubkey,
    accounts: admin_program_freeze::Accounts<Pubkey>,
    params: admin_program_freeze::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::AdminProgramFreeze as u8,
        params,
    )
}

#[allow(missing_docs)]
pub fn admin_renounce(
    program_id: Pubkey,
    accounts: admin_renounce::Accounts<Pubkey>,
    params: admin_renounce::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::AdminRenounce as u8, params)
}

#[allow(missing_docs)]
pub fn admin_change_freeze_authority(
    program_id: Pubkey,
    accounts: admin_change_freeze_authority::Accounts<Pubkey>,
    params: admin_change_freeze_authority::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::AdminChangeFreezeAuthority as u8,
        params,
    )
}

#[allow(missing_docs)]
pub fn create_royalty_account(
    program_id: Pubkey,
    accounts: create_royalty_account::Accounts<Pubkey>,
    params: create_royalty_account::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::CreateRoyaltyAccount as u8,
        params,
    )
}

#[allow(missing_docs)]
pub fn close_royalty_account(
    program_id: Pubkey,
    accounts: close_royalty_account::Accounts<Pubkey>,
    params: close_royalty_account::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::CloseRoyaltyAccount as u8,
        params,
    )
}

#[allow(missing_docs)]
pub fn cpi_claim_rewards(
    program_id: Pubkey,
    accounts: cpi_claim_rewards::Accounts<Pubkey>,
    params: cpi_claim_rewards::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::CpiClaimRewards as u8, params)
}

#[allow(missing_docs)]
pub fn cpi_claim_bond_v2_rewards(
    program_id: Pubkey,
    accounts: cpi_claim_bond_v2_rewards::Accounts<Pubkey>,
    params: cpi_claim_bond_v2_rewards::Params,
) -> Instruction {
    accounts.get_instruction(
        program_id,
        ProgramInstruction::CpiClaimBondV2Rewards as u8,
        params,
    )
}