//! Execute the token transfer after the unstake outbounding period
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use spl_token::instruction::transfer;

use crate::{
    state::StakePoolHeader,
    utils::{check_account_key, check_account_owner, check_signer},
};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::error::AccessError;
use crate::state::{StakeAccount, StakePool, Tag};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `unstake` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `unstake` instruction
pub struct Accounts<'a, T> {
    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The owner of the stake account
    #[cons(signer)]
    pub owner: &'a T,

    /// The destination of the staked tokens
    #[cons(writable)]
    pub destination_token: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The stake pool vault
    #[cons(writable)]
    pub vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_account: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            destination_token: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.stake_account,
            program_id,
            AccessError::WrongStakeAccountOwner,
        )?;
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.destination_token,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        check_account_owner(
            accounts.vault,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;

        // Check signer
        check_signer(accounts.owner, AccessError::StakeAccountOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_execute_unstake(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let stake_pool = StakePool::get_checked(accounts.stake_pool, Tag::StakePool)?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;
    let current_time = Clock::get()?.unix_timestamp;

    check_account_key(
        accounts.owner,
        &stake_account.owner,
        AccessError::StakeAccountOwnerMismatch,
    )?;
    check_account_key(
        accounts.stake_pool,
        &stake_account.stake_pool,
        AccessError::StakePoolMismatch,
    )?;
    check_account_key(
        accounts.vault,
        &Pubkey::new(&stake_pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    let pending_request = stake_account.pop_unstake_request()?;

    if current_time
        .checked_sub(pending_request.time)
        .ok_or(AccessError::Overflow)?
        < stake_pool.header.unstake_period
        || pending_request.amount == 0
    {
        return Err(AccessError::CannotUnstake.into());
    }

    // Transfer tokens
    let signer_seeds: &[&[u8]] = &[
        StakePoolHeader::SEED,
        &stake_pool.header.owner.clone(),
        &[stake_pool.header.nonce],
    ];
    let transfer_instruction = transfer(
        &spl_token::ID,
        accounts.vault.key,
        accounts.destination_token.key,
        accounts.stake_pool.key,
        &[],
        pending_request.amount,
    )?;

    drop(stake_pool);

    invoke_signed(
        &transfer_instruction,
        &[
            accounts.spl_token_program.clone(),
            accounts.vault.clone(),
            accounts.destination_token.clone(),
            accounts.stake_pool.clone(),
        ],
        &[signer_seeds],
    )?;

    // Save states
    stake_account.save(&mut accounts.stake_account.data.borrow_mut())?;

    Ok(())
}
