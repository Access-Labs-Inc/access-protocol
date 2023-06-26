//! Mint subscription
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey};
use solana_program::program::invoke_signed;
use solana_program::program_pack::Pack;
use spl_token::instruction::transfer;

use crate::{
    utils::{check_account_key, check_account_owner, check_signer},
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `mint_subscription` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `mint_subscription` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state_account: next_account_info(accounts_iter)?,
        };
        Ok(accounts)
    }
}

pub fn process_mint_subscription(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    msg!("Minting subscription NFT");
    // todo real logic
    Ok(())
}
