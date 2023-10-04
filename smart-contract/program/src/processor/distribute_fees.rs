//! Distribute fees to the recipients
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;
use spl_token::state::Account;

use crate::{
    state::MIN_DISTRIBUTE_AMOUNT,
    utils::{assert_valid_vault, check_account_key, check_account_owner},
};
use crate::error::AccessError;
use crate::instruction::ProgramInstruction::DistributeFees;
use crate::state::CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `distribute_fees` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `distribute_fees` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The central state ATA
    #[cons(writable)]
    pub central_state_vault: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The mint address of the ACS token
    #[cons(writable)]
    pub mint: &'a T,

    /// The token accounts to distribute the fees to
    #[cons(writable)]
    pub token_accounts: &'a [T],
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state: next_account_info(accounts_iter)?,
            central_state_vault: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            token_accounts: accounts_iter.as_slice(),
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.central_state,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(
            accounts.central_state_vault,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        check_account_owner(
            accounts.mint,
            &spl_token::ID,
            AccessError::WrongOwner,
        )?;
        for token_account in accounts.token_accounts {
            check_account_owner(
                token_account,
                &spl_token::ID,
                AccessError::WrongTokenAccountOwner,
            )?;
        }

        // Check signer
        Ok(accounts)
    }
}

pub fn process_distribute_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&DistributeFees)?;
    assert_valid_vault(accounts.central_state_vault, accounts.central_state.key)?;

    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;

    // check recipient count
    if accounts.token_accounts.len() != central_state.recipients.len() {
        msg!("Invalid count of the token accounts");
        return Err(AccessError::InvalidTokenAccount.into());
    }

    // Distribute
    let central_state_vault = Account::unpack(&accounts.central_state_vault.data.borrow())?;
    let total_balance = central_state_vault.amount;
    msg!("Balance to distribute: {}", total_balance);
    let mut remaining_balance = total_balance;

    // This covers us against someone calling it too often and thereby burning too many excess tokens
    if total_balance < MIN_DISTRIBUTE_AMOUNT {
        msg!("Not enough tokens to distribute: {}", total_balance);
        return Err(AccessError::InvalidAmount.into());
    }

    for (i, (token_account, recipient)) in accounts
        .token_accounts
        .iter()
        .zip(central_state.recipients.iter())
        .enumerate()
    {
        if *token_account.owner != recipient.owner {
            msg!("Invalid ordering of the token accounts");
            return Err(AccessError::InvalidTokenAccount.into());
        }
        let amount = total_balance
            .checked_mul(recipient.percentage)
            .ok_or(AccessError::Overflow)?
            .checked_div(100)
            .ok_or(AccessError::Overflow)?;
        if amount == 0 {
            msg!("Skipping zero amount for recipient with index {}", i);
            continue;
        }
        let ix = spl_token::instruction::transfer(
            &spl_token::ID,
            accounts.central_state_vault.key,
            token_account.key,
            accounts.central_state.key,
            &[],
            amount,
        )
        .unwrap();
        invoke_signed(
            &ix,
            &[
                accounts.spl_token_program.clone(),
                accounts.central_state_vault.clone(),
                token_account.clone(),
                accounts.central_state.clone(),
            ],
            &[&[&program_id.to_bytes(), &[central_state.bump_seed]]],
        )?;
        remaining_balance = remaining_balance
            .checked_sub(amount)
            .ok_or(AccessError::Overflow)?;
    }

    // There is almost always something to burn due to rounding down.
    // However, this will be an insignificantly small amount (<= 1 ^ -5 ACS) if the sum of the percentages is 100.
    if remaining_balance > 0 {
        let burn_instruction = spl_token::instruction::burn(
            &spl_token::ID,
            accounts.central_state_vault.key,
            accounts.mint.key,
            accounts.central_state.key,
            &[],
            remaining_balance,
        )?;
        invoke_signed(
            &burn_instruction,
            &[
                accounts.central_state_vault.clone(),
                accounts.mint.clone(),
                accounts.central_state.clone(),
                accounts.central_state.clone(),
            ],
            &[&[&program_id.to_bytes(), &[central_state.bump_seed]]],
        )?;
        msg!("Burned {} tokens", remaining_balance);
    }

    central_state.last_fee_distribution_time = Clock::get()?.unix_timestamp;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;
    Ok(())
}
