use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::MediaInstruction;

pub mod change_inflation;
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
            }
            MediaInstruction::CreateStakePool(params) => {
                msg!("Instruction: Create stake pool");
            }
            MediaInstruction::CreateStakeAccount(params) => {
                msg!("Instruction: Create stake account");
            }
            MediaInstruction::Stake(params) => {
                msg!("Instruction: Stake");
            }
            MediaInstruction::Unstake(params) => {
                msg!("Instruction: Unstake");
            }
            MediaInstruction::ClaimRewards => {
                msg!("Instruction: Claim rewards");
            }
            MediaInstruction::CloseStakePool => {
                msg!("Instruction: Close stake pool");
            }
            MediaInstruction::CloseStakeAccount => {
                msg!("Instruction: Close stake account");
            }
            MediaInstruction::ChangeInflationRate(params) => {
                msg!("Instruction: Change inflation rate");
            }
        }

        Ok(())
    }
}
