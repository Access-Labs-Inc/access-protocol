//! Unstake
use borsh::{BorshDeserialize, BorshSerialize};
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
use crate::state::{StakeAccount, StakePool};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    // Amount to unstake
    pub amount: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The owner of the stake account
    #[cons(writable, signer)] //TODO rm writable
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
        )?; // Not strictly necessary, done by spl token
        check_account_owner(
            accounts.vault,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?; // Not strictly necessary, done by spl token

        // Check signer
        check_signer(accounts.owner, AccessError::StakeAccountOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let Params { amount } = params;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool)?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;

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

    // Update stake account
    stake_account.withdraw(amount)?;
    stake_pool.header.withdraw(amount)?;

    // Save states
    stake_account.save(&mut accounts.stake_account.data.borrow_mut());

    // Transfer tokens
    let signer_seeds: &[&[u8]] = &[
        StakePoolHeader::SEED.as_bytes(),
        &stake_pool.header.owner.clone(),
        &stake_pool.header.rewards_destination.clone(),
        &[stake_pool.header.nonce],
    ];
    let transfer_instruction = transfer(
        &spl_token::ID,
        accounts.vault.key,
        accounts.destination_token.key,
        accounts.stake_pool.key,
        &[],
        amount,
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

    Ok(())
}
