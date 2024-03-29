//! Setup fee split

use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::AdminSetupFeeSplit;
use crate::state::{
    FeeRecipient, MAX_FEE_RECIPIENTS, MAX_FEE_SPLIT_SETUP_DELAY,
};
use crate::state::CentralStateV2;
use crate::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `admin_setup_fee_split` instruction
pub struct Params {
    pub recipients: Vec<FeeRecipient>,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `admin_setup_fee_split` instruction
pub struct Accounts<'a, T> {
    /// The central state authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            authority: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;

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
    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&AdminSetupFeeSplit)?;

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    // Check if right number of recipients
    if recipients.len() > MAX_FEE_RECIPIENTS {
        msg!("Too many recipients");
        return Err(AccessError::TooManyRecipients.into());
    }

    // Check recipients
    let mut percentage_sum: u64 = 0;
    recipients.iter().try_for_each(|r| -> ProgramResult {
        if r.percentage == 0 {
            msg!("Recipient percentage 0 not allowed");
            return Err(AccessError::InvalidPercentages.into());
        }
        percentage_sum = percentage_sum
            .checked_add(r.percentage)
            .ok_or(AccessError::Overflow)?;
        if percentage_sum > 100 {
            msg!("Percentages add up to more than 100");
            return Err(AccessError::InvalidPercentages.into());
        }
        Ok(())
    })?;

    // The recipient setup must be done within 5 minutes after the fee distribution
    let current_time = Clock::get()?.unix_timestamp as u64;
    if current_time - central_state.last_fee_distribution_time as u64 > MAX_FEE_SPLIT_SETUP_DELAY {
        msg!("Delay between fee distribution and fee split setup too long");
        return Err(AccessError::DelayTooLong.into());
    }

    central_state.recipients = recipients;

    // replace the recipients
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;
    Ok(())
}
