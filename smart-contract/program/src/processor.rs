use borsh::BorshDeserialize;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::ProgramInstruction;

pub mod activate_stake_pool;
pub mod add_to_bond_v2;
pub mod admin_freeze;
pub mod admin_mint;
pub mod admin_setup_fee_split;
pub mod change_central_state_authority;
pub mod change_inflation;
pub mod change_pool_minimum;
pub mod change_pool_multiplier;
pub mod claim_bond;
pub mod claim_bond_rewards;
pub mod claim_bond_v2_rewards;
pub mod claim_pool_rewards;
pub mod claim_rewards;
pub mod close_stake_account;
pub mod close_stake_pool;
pub mod crank;
pub mod create_bond;
pub mod create_bond_v2;
pub mod create_central_state;
pub mod create_stake_account;
pub mod create_stake_pool;
pub mod distribute_fees;
pub mod edit_metadata;
pub mod sign_bond;
pub mod stake;
pub mod unlock_bond_tokens;
pub mod unlock_bond_v2;
pub mod unstake;
pub mod admin_set_protocol_fee;
pub mod migrate_central_state_v2;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = FromPrimitive::from_u8(instruction_data[0])
            .ok_or(ProgramError::InvalidInstructionData)?;
        let instruction_data = &instruction_data[1..];
        msg!("Instruction unpacked");

        match instruction {
            ProgramInstruction::CreateCentralState => {
                msg!("Instruction: Create central state");
                let params = create_central_state::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_central_state::process_create_central_state(program_id, accounts, params)?;
            }
            ProgramInstruction::CreateStakePool => {
                msg!("Instruction: Create stake pool");
                let params = create_stake_pool::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_stake_pool::process_create_stake_pool(program_id, accounts, params)?;
            }
            ProgramInstruction::ActivateStakePool => {
                msg!("Instruction: Activate stake pool");
                activate_stake_pool::process_activate_stake_pool(program_id, accounts)?;
            }
            ProgramInstruction::CreateStakeAccount => {
                msg!("Instruction: Create stake account");
                let params = create_stake_account::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_stake_account::process_create_stake_account(program_id, accounts, params)?;
            }
            ProgramInstruction::Stake => {
                msg!("Instruction: Stake");
                let params = stake::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                stake::process_stake(program_id, accounts, params)?;
            }
            ProgramInstruction::Unstake => {
                msg!("Instruction: Unstake");
                let params = unstake::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                unstake::process_unstake(program_id, accounts, params)?;
            }
            ProgramInstruction::ClaimPoolRewards => {
                msg!("Instruction: Claim pool rewards");
                let params = claim_pool_rewards::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_pool_rewards::process_claim_pool_rewards(program_id, accounts, params)?;
            }
            ProgramInstruction::ClaimRewards => {
                msg!("Instruction: Claim rewards");
                let params = claim_rewards::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_rewards::process_claim_rewards(program_id, accounts, params)?;
            }
            ProgramInstruction::Crank => {
                msg!("Instruction: Crank");
                let params = crank::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                crank::process_crank(program_id, accounts, params)?;
            }
            ProgramInstruction::CloseStakePool => {
                msg!("Instruction: Close stake pool");
                let params = close_stake_pool::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                close_stake_pool::process_close_stake_pool(program_id, accounts, params)?;
            }
            ProgramInstruction::CloseStakeAccount => {
                msg!("Instruction: Close stake account");
                close_stake_account::process_close_stake_account(program_id, accounts)?;
            }
            ProgramInstruction::ChangeInflation => {
                msg!("Instruction: Change inflation rate");
                let params = change_inflation::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                change_inflation::process_change_inflation(program_id, accounts, params)?;
            }
            ProgramInstruction::CreateBond => {
                msg!("Instruction: create bond");
                let params = create_bond::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_bond::process_create_bond(program_id, accounts, params)?;
            }
            ProgramInstruction::SignBond => {
                msg!("Instruction: sign bond");
                let params = sign_bond::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                sign_bond::process_sign_bond(program_id, accounts, params)?;
            }
            ProgramInstruction::UnlockBondTokens => {
                msg!("Instruction: unlock bond token");
                let params = unlock_bond_tokens::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                unlock_bond_tokens::process_unlock_bond_tokens(program_id, accounts, params)?;
            }
            ProgramInstruction::ClaimBond => {
                msg!("Instruction: claim bond");
                let params = claim_bond::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_bond::process_claim_bond(program_id, accounts, params)?;
            }
            ProgramInstruction::ClaimBondRewards => {
                msg!("Instruction: claim bond rewards");
                let params = claim_bond_rewards::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_bond_rewards::process_claim_bond_rewards(program_id, accounts, params)?;
            }
            ProgramInstruction::ChangePoolMinimum => {
                msg!("Instruction: Change pool minimum");
                let params = change_pool_minimum::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                change_pool_minimum::process_change_pool_minimum(program_id, accounts, params)?;
            }
            ProgramInstruction::AdminMint => {
                msg!("Instruction: Mint ACCESS tokens");
                let params = admin_mint::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                admin_mint::process_admin_mint(program_id, accounts, params)?;
            }
            ProgramInstruction::AdminFreeze => {
                msg!("Instruction: Admin freeze");
                admin_freeze::process_admin_freeze(program_id, accounts)?;
            }
            ProgramInstruction::ChangePoolMultiplier => {
                msg!("Instruction: Change pool multiplier");
                let params = change_pool_multiplier::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                change_pool_multiplier::process_change_pool_multiplier(
                    program_id, accounts, params,
                )?;
            }
            ProgramInstruction::ChangeCentralStateAuthority => {
                msg!("Instruction: Change central state authority");
                let params =
                    change_central_state_authority::Params::try_from_slice(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;
                change_central_state_authority::process_change_central_state_auth(
                    program_id, accounts, params,
                )?;
            }
            ProgramInstruction::EditMetadata => {
                msg!("Instruction: Edit Access token metadata");
                let params = edit_metadata::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                edit_metadata::process_edit_metadata(program_id, accounts, params)?;
            }
            ProgramInstruction::CreateBondV2 => {
                msg!("Instruction: Create bond V2");
                let params = create_bond_v2::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_bond_v2::process_create_bond_v2(program_id, accounts, params)?;
            }
            ProgramInstruction::AddToBondV2 => {
                msg!("Instruction: Add to bond V2");
                let params = add_to_bond_v2::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                add_to_bond_v2::process_add_to_bond_v2(program_id, accounts, params)?;
            }
            ProgramInstruction::ClaimBondV2Rewards => {
                msg!("Instruction: Claim bond V2 rewards");
                let params = claim_bond_v2_rewards::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_bond_v2_rewards::process_claim_bond_v2_rewards(program_id, accounts, params)?;
            }
            ProgramInstruction::UnlockBondV2 => {
                msg!("Instruction: Unlock bond V2");
                let params = unlock_bond_v2::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                unlock_bond_v2::process_unlock_bond_v2(program_id, accounts, params)?;
            }
            ProgramInstruction::AdminSetupFeeSplit => {
                msg!("Instruction: Admin setup fee split");
                let params = admin_setup_fee_split::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                admin_setup_fee_split::process_admin_setup_fee_split(program_id, accounts, params)?;
            }
            ProgramInstruction::DistributeFees => {
                msg!("Instruction: Distribute fees");
                let params = distribute_fees::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                distribute_fees::process_distribute_fees(program_id, accounts, params)?;
            }
            ProgramInstruction::AdminSetProtocolFee => {
                msg!("Instruction: Admin set protocol fee");
                let params = admin_set_protocol_fee::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                admin_set_protocol_fee::process_admin_set_protocol_fee(program_id, accounts, params)?;
            }
            ProgramInstruction::MigrateCentralStateV2 => {
                msg!("Instruction: Migrate central state V2");
                let params = migrate_central_state_v2::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                migrate_central_state_v2::process_migrate_central_state_v2(
                    program_id, accounts, params,
                )?;
            }
        }

        Ok(())
    }
}
