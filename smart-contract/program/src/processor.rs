use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::MediaInstruction;

pub mod change_inflation;
pub mod claim_pool_rewards;
pub mod claim_rewards;
pub mod close_stake_account;
pub mod close_stake_pool;
pub mod crank;
pub mod create_central_state;
pub mod create_stake_account;
pub mod create_stake_pool;
pub mod stake;
pub mod unstake;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = MediaInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        msg!("Instruction unpacked");

        match instruction {
            MediaInstruction::CreateCentralState(params) => {
                msg!("Instruction: Create central state");
                create_central_state::process_create_central_state(program_id, accounts, params)?;
            }
            MediaInstruction::CreateStakePool(params) => {
                msg!("Instruction: Create stake pool");
                create_stake_pool::process_create_stake_pool(program_id, accounts, params)?;
            }
            MediaInstruction::CreateStakeAccount(params) => {
                msg!("Instruction: Create stake account");
                create_stake_account::process_create_stake_account(program_id, accounts, params)?;
            }
            MediaInstruction::Stake(params) => {
                msg!("Instruction: Stake");
                stake::process_stake(program_id, accounts, params)?;
            }
            MediaInstruction::Unstake(params) => {
                msg!("Instruction: Unstake");
                unstake::process_unstake(program_id, accounts, params)?;
            }
            MediaInstruction::ClaimPoolRewards(params) => {
                msg!("Instruction: Claim pool rewards");
                claim_pool_rewards::process_claim_pool_rewards(program_id, accounts, params)?;
            }
            MediaInstruction::ClaimRewards(params) => {
                msg!("Instruction: Claim rewards");
                claim_rewards::process_claim_rewards(program_id, accounts, params)?;
            }
            MediaInstruction::Crank(params) => {
                msg!("Instruction: Crank");
                crank::process_crank(program_id, accounts, params)?;
            }
            MediaInstruction::CloseStakePool(params) => {
                msg!("Instruction: Close stake pool");
                close_stake_pool::process_close_stake_pool(program_id, accounts, params)?;
            }
            MediaInstruction::CloseStakeAccount(params) => {
                msg!("Instruction: Close stake account");
                close_stake_account::process_close_stake_account(program_id, accounts, params)?;
            }
            MediaInstruction::ChangeInflation(params) => {
                msg!("Instruction: Change inflation rate");
                change_inflation::process_change_inflation(program_id, accounts, params)?;
            }
        }

        Ok(())
    }
}
