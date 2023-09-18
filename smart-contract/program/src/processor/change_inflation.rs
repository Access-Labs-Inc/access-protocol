//! Change central state inflation
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey};
use solana_program::program_pack::Pack;

use crate::{error::AccessError};
use crate::instruction::ProgramInstruction::ChangeInflation;
use crate::utils::{check_account_key, check_account_owner, check_signer};
use crate::state:: CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `change_inflation` instruction
pub struct Params {
    // The new daily inflation token amount
    pub daily_inflation: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `change_inflation` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The central state account authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The mint address of the ACCESS token
    pub mint: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.mint, &spl_token::ID, AccessError::WrongOwner)?;

        // Check signer
        check_signer(
            accounts.authority,
            AccessError::CentralStateAuthorityMustSign,
        )?;

        Ok(accounts)
    }
}

pub fn process_change_inflation(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(ChangeInflation)?;

    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;

    let token_mint = spl_token::state::Mint::unpack_from_slice(&accounts.mint.data.clone().borrow_mut())?;

    let supply = token_mint.supply;
    let annual_inflation = params.daily_inflation * 365;
    if annual_inflation > supply {
        msg!("Inflation is too high, maximum annual {}, requested {}", supply, annual_inflation);
        return Err(AccessError::InvalidAmount.into());
    }

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    central_state.daily_inflation = params.daily_inflation;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
