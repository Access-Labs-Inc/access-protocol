//! Create central state
use std::mem::size_of;

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
use solana_program::clock::Clock;
use solana_program::program_pack::Pack;
use solana_program::sysvar::Sysvar;
use spl_token::state::Account;

use crate::{cpi::Cpi, error::AccessError};
use crate::state::{CentralState, FeeRecipient, FeeSplit, MAX_FEE_RECIPIENTS, MAX_FEE_SPLIT_SETUP_DELAY};
use crate::utils::{check_account_key, check_account_owner, check_signer};
use crate::utils::assert_valid_vault;

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
    pub fee_split_pda: &'a T,

    /// The stake pool vault account
    pub fee_split_ata: &'a T,

    /// The account of the central state
    pub central_state: &'a T,

    /// The system program account
    pub system_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>], program_id: &Pubkey) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            authority: next_account_info(accounts_iter)?,
            fee_split_pda: next_account_info(accounts_iter)?,
            fee_split_ata: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
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

        // Check signer
        check_signer(
            accounts.authority,
            AccessError::CentralStateAuthorityMustSign,
        )?;

        Ok(accounts)
    }
}

pub fn process_admin_setup_fee_split(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params { recipients } = params;
    let accounts = Accounts::parse(accounts, program_id)?;
    let (fee_split_pda, bump_seed) = FeeSplit::find_key(program_id);
    let mut central_state = CentralState::from_account_info(accounts.central_state)?;

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;
    assert_valid_vault(accounts.fee_split_ata, &fee_split_pda)?;
    check_account_key(
        accounts.fee_split_pda,
        &fee_split_pda,
        AccessError::AccountNotDeterministic,
    )?;

    let fee_split_ata = Account::unpack(&accounts.fee_split_ata.data.borrow())?;
    if fee_split_ata.mint != central_state.token_mint {
        return Err(AccessError::WrongMint.into());
    }
    if &fee_split_ata.owner != accounts.fee_split_pda.key {
        return Err(AccessError::WrongOwner.into());
    }

    // Check if right number of recipients
    if recipients.len() > MAX_FEE_RECIPIENTS as usize {
        msg!("Too many recipients");
        return Err(AccessError::TooManyRecipients.into());
    }
    if recipients.len() == 0 {
        msg!("No recipients");
        return Err(AccessError::NoRecipients.into());
    }

    // Check recipients
    let mut percentage_sum: u64 = 0;
    recipients.iter().try_for_each(|r| -> ProgramResult {
        if r.percentage == 0 {
            msg!("Recipient percentage 0 not allowed");
            return Err(AccessError::InvalidPercentages.into());
        }
        percentage_sum = percentage_sum.checked_add(r.percentage).ok_or(AccessError::Overflow)?;
        if percentage_sum > 100 {
            msg!("Percentages add up to more than 100");
            return Err(AccessError::InvalidPercentages.into());
        }
        Ok(())
    })?;


    let mut fee_split: FeeSplit;
    if accounts.fee_split_pda.data_is_empty() {
        msg!("Creating Fee split account");
        fee_split = FeeSplit::new(
            bump_seed,
            recipients,
        )?;

        Cpi::create_account(
            program_id,
            accounts.system_program,
            accounts.authority,
            accounts.fee_split_pda,
            &[FeeSplit::SEED, &program_id.to_bytes(), &[bump_seed]],
            fee_split.borsh_len() + size_of::<FeeRecipient>() * MAX_FEE_RECIPIENTS as usize,
        )?;
    } else {
        check_account_owner(
            accounts.fee_split_pda,
            program_id,
            AccessError::WrongOwner,
        )?;
        fee_split = FeeSplit::from_account_info(accounts.fee_split_pda)?;

        let current_time = Clock::get()?.unix_timestamp as u64;
        if current_time - fee_split.last_distribution_time as u64 > MAX_FEE_SPLIT_SETUP_DELAY {
            msg!("Delay between fee distribution and fee split setup too long");
            return Err(AccessError::DelayTooLong.into());
        }

        fee_split.recipients = recipients.clone();
    }

    // replace the recipients
    fee_split.save(&mut accounts.fee_split_pda.data.borrow_mut())?;
    Ok(())
}
