//! Create central state
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use std::mem::size_of;
use crate::{state::StakePool, utils::assert_valid_vault};
use crate::{cpi::Cpi, error::AccessError};
use crate::state::{FeeSplit, CentralState, FeeRecipient, MAX_FEE_RECIPIENTS};
use crate::utils::{check_account_key, check_account_owner};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_central_state` instruction
pub struct Params {
    pub recipients: Vec<FeeRecipient>,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create_central_state` instruction
pub struct Accounts<'a, T> {
    /// The central state authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The fee split account
    #[cons(writable)]
    pub fee_spit_account: &'a T,

    /// The account of the central state
    pub central_state: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The stake pool vault account
    pub vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>], program_id: &Pubkey) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            authority: next_account_info(accounts_iter)?,
            fee_spit_account: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        };

        // Check keys - todo more
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

        // Check ownership
        check_account_owner(
            accounts.central_state,
            program_id,
            AccessError::WrongOwner,
        )?;

        Ok(accounts)
    }
}

pub fn process_admin_setup_fee_split(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let (fee_split_pda, bump_seed) = FeeSplit::find_key(program_id);
    let mut central_state = CentralState::from_account_info(accounts.central_state)?;

    assert_valid_vault(accounts.vault, &fee_split_pda)?;
    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    check_account_key(
        accounts.fee_spit_account,
        &fee_split_pda,
        AccessError::AccountNotDeterministic,
    )?;
    // Check if more recipients than allowed
        if params.recipients.len() > MAX_FEE_RECIPIENTS as usize {
            msg!("Too many recipients");
            return Err(AccessError::TooManyRecipients.into());
        }
    // Check if the percentages add up to 100
    // todo maybe safe math
                if params.recipients.iter().map(|r| r.percentage).sum::<u64>() != 100 {
                    msg!("Percentages don't add up to 100");
                    return Err(AccessError::InvalidPercentages.into());
                }

    // todo Check that the recipients are valid ATAs for our mint

    let mut fee_split:FeeSplit;
    if accounts.fee_spit_account.data_is_empty() {
        msg!("Creating Fee split account");
        fee_split = FeeSplit::new(
            bump_seed,
            params.recipients,
        );

        Cpi::create_account(
            program_id,
            accounts.system_program,
            accounts.authority,
            accounts.fee_spit_account,
            &[FeeSplit::SEED, &program_id.to_bytes(), &[bump_seed]],
            fee_split.borsh_len() + size_of::<FeeRecipient>() * MAX_FEE_RECIPIENTS as usize
        )?;

    } else {
        fee_split = FeeSplit::from_account_info(accounts.fee_spit_account)?;
        fee_split.recipients = params.recipients.clone();
    }

    // replace the recipients
    fee_split.save(&mut accounts.fee_spit_account.data.borrow_mut())?;
    Ok(())
}
