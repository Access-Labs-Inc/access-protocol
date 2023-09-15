//! Close a stake pool
//! This instruction can be used to close an empty stake pool and collect the lamports
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account;
use crate::state:: CentralStateV2;

use crate::error::AccessError;
use crate::state::MAX_FEE_RECIPIENTS;
use crate::{
    state::MIN_DISTRIBUTE_AMOUNT,
    utils::{check_account_key, check_account_owner, check_signer, assert_valid_vault},
};
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;
use crate::instruction::ProgramInstruction::DistributeFees;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `close_stake_pool` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `close_stake_pool` instruction
pub struct Accounts<'a, T> {
    // todo comments
    #[cons(signer)]
    pub fee_payer: &'a T,

    #[cons(writable)]
    pub central_state: &'a T,

    #[cons(writable)]
    pub central_state_vault: &'a T,

    pub spl_token_program: &'a T,

    #[cons(writable)]
    pub mint: &'a T,
    /// Pool vault
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
            fee_payer: next_account_info(accounts_iter)?,
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
        for token_account in accounts.token_accounts {
            check_account_owner(
                token_account,
                &spl_token::ID,
                AccessError::WrongTokenAccountOwner,
            )?;
        }

        // todo is this needed?
        // Check signer
        check_signer(accounts.fee_payer, AccessError::StakeAccountOwnerMustSign)?;
        Ok(accounts)
    }
}

pub fn process_distribute_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    if accounts.token_accounts.len() > MAX_FEE_RECIPIENTS {
        msg!("Too many token accounts to distribute to");
        return Err(AccessError::InvalidTokenAccount.into());
    }

    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(DistributeFees)?;
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

    // check ATA mints
    let central_state_vault = Account::unpack(&accounts.central_state_vault.data.borrow())?;
    if central_state_vault.mint != central_state.token_mint {
        return Err(AccessError::WrongMint.into());
    }

    for token_account in accounts.token_accounts {
        let token_account = Account::unpack(&token_account.data.borrow())?;
        if token_account.mint != central_state.token_mint {
            return Err(AccessError::WrongMint.into());
        }
    }

    // Distribute
    let total_balance = central_state_vault.amount;
    msg!("Balance to distribute: {}", total_balance);
    let mut remaining_balance = total_balance;

    // This covers us against someone calling it too often and thereby burning too many excess tokens
    if total_balance < MIN_DISTRIBUTE_AMOUNT {
        msg!("Not enough tokens to distribute: {}", total_balance);
        return Err(AccessError::InvalidAmount.into());
    }

    for (token_account, recipient) in accounts
        .token_accounts
        .iter()
        .zip(central_state.recipients.iter())
    {
        let recipient_ata = recipient.ata(&central_state.token_mint);
        if *token_account.key != recipient_ata {
            msg!("Invalid ordering of the token accounts");
            return Err(AccessError::InvalidTokenAccount.into());
        }
        let amount = total_balance
            .checked_mul(recipient.percentage)
            .ok_or(AccessError::Overflow)?
            .checked_div(100)
            .ok_or(AccessError::Overflow)?;
        if amount == 0 {
            msg!("Skipping zero amount for {}", recipient_ata);
            continue;
        }
        let ix = spl_token::instruction::transfer(
            &spl_token::ID,
            accounts.central_state_vault.key,
            &recipient_ata,
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
