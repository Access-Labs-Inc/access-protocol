use borsh::BorshDeserialize;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::MediaInstruction;

pub mod change_inflation;
pub mod claim_bond;
pub mod claim_bond_rewards;
pub mod claim_pool_rewards;
pub mod claim_rewards;
pub mod close_stake_account;
pub mod close_stake_pool;
pub mod crank;
pub mod create_bond;
pub mod create_central_state;
pub mod create_stake_account;
pub mod create_stake_pool;
pub mod sign_bond;
pub mod stake;
pub mod unlock_bond_tokens;
pub mod unstake;

// TODO change to mint_to instead of transfers
// Allow mint ix (or use bond?)

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
            MediaInstruction::CreateCentralState => {
                msg!("Instruction: Create central state");
                let params = create_central_state::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_central_state::process_create_central_state(program_id, accounts, params)?;
            }
            MediaInstruction::CreateStakePool => {
                msg!("Instruction: Create stake pool");
                let params = create_stake_pool::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_stake_pool::process_create_stake_pool(program_id, accounts, params)?;
            }
            MediaInstruction::CreateStakeAccount => {
                msg!("Instruction: Create stake account");
                let params = create_stake_account::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_stake_account::process_create_stake_account(program_id, accounts, params)?;
            }
            MediaInstruction::Stake => {
                msg!("Instruction: Stake");
                let params = stake::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                stake::process_stake(program_id, accounts, params)?;
            }
            MediaInstruction::Unstake => {
                msg!("Instruction: Unstake");
                let params = unstake::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                unstake::process_unstake(program_id, accounts, params)?;
            }
            MediaInstruction::ClaimPoolRewards => {
                msg!("Instruction: Claim pool rewards");
                let params = claim_pool_rewards::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_pool_rewards::process_claim_pool_rewards(program_id, accounts, params)?;
            }
            MediaInstruction::ClaimRewards => {
                msg!("Instruction: Claim rewards");
                let params = claim_rewards::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_rewards::process_claim_rewards(program_id, accounts, params)?;
            }
            MediaInstruction::Crank => {
                msg!("Instruction: Crank");
                let params = crank::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                crank::process_crank(program_id, accounts, params)?;
            }
            MediaInstruction::CloseStakePool => {
                msg!("Instruction: Close stake pool");
                let params = close_stake_pool::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                close_stake_pool::process_close_stake_pool(program_id, accounts, params)?;
            }
            MediaInstruction::CloseStakeAccount => {
                msg!("Instruction: Close stake account");
                let params = close_stake_account::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                close_stake_account::process_close_stake_account(program_id, accounts, params)?;
            }
            MediaInstruction::ChangeInflation => {
                msg!("Instruction: Change inflation rate");
                let params = change_inflation::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                change_inflation::process_change_inflation(program_id, accounts, params)?;
            }
            MediaInstruction::CreateBond => {
                msg!("Instruction: create bond");
                let params = create_bond::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_bond::process_create_bond(program_id, accounts, params)?;
            }
            MediaInstruction::SignBond => {
                msg!("Instruction: sign bond");
                let params = sign_bond::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                sign_bond::process_sign_bond(program_id, accounts, params)?;
            }
            MediaInstruction::UnlockBondTokens => {
                msg!("Instruction: unlock bond token");
                let params = unlock_bond_tokens::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                unlock_bond_tokens::process_unlock_bond_tokens(program_id, accounts, params)?;
            }
            MediaInstruction::ClaimBond => {
                msg!("Instruction: claim bond");
                let params = claim_bond::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_bond::process_claim_bond(program_id, accounts, params)?;
            }
            MediaInstruction::ClaimBondRewards => {
                msg!("Instruction: claim bond rewards");
                let params = claim_bond_rewards::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                claim_bond_rewards::process_claim_bond_rewards(program_id, accounts, params)?;
            }
        }

        Ok(())
    }
}
