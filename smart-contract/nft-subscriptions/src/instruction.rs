pub use crate::processor::{
    mint_subscription,
};
use bonfida_utils::InstructionsAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{instruction::Instruction, pubkey::Pubkey};

#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Mint a subscription NFT
    MintSubscription,
}

#[allow(missing_docs)]
pub fn mint_subscription(
    program_id: Pubkey,
    accounts: mint_subscription::Accounts<Pubkey>,
    params: mint_subscription::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::MintSubscription as u8, params)
}