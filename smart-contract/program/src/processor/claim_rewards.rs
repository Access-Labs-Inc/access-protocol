//! Claim rewards of a stake account
use crate::error::MediaError;
use crate::state::{CentralState, StakeAccount, StakePool, STAKER_MULTIPLIER};
use crate::utils::{
    calc_previous_balances_and_inflation, check_account_key, check_account_owner, check_signer,
    safe_downcast,
};
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::invoke_signed;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::{instruction::transfer, state::Mint};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The owner of the stake account
    #[cons(writable, signer)]
    pub owner: &'a T,

    /// The rewards destination
    #[cons(writable)]
    pub rewards_destination: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The mint address of the ACCESS token
    pub mint: &'a T,
    #[cons(writable)]

    /// The central vault account
    pub central_vault: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_pool: next_account_info(accounts_iter)?,
            stake_account: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            rewards_destination: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            central_vault: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            MediaError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.stake_pool,
            program_id,
            MediaError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.stake_account,
            program_id,
            MediaError::WrongStakeAccountOwner,
        )?;
        check_account_owner(
            accounts.rewards_destination,
            &spl_token::ID,
            MediaError::WrongOwner,
        )?;
        check_account_owner(accounts.central_state, program_id, MediaError::WrongOwner)?;
        check_account_owner(accounts.mint, &spl_token::ID, MediaError::WrongOwner)?;
        check_account_owner(
            accounts.central_vault,
            &spl_token::ID,
            MediaError::WrongOwner,
        )?;

        // Check signer
        check_signer(accounts.owner, MediaError::StakePoolOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_claim_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let current_time = Clock::get().unwrap().unix_timestamp;

    let central_state = CentralState::from_account_info(accounts.central_state)?;
    let stake_pool = StakePool::get_checked(accounts.stake_pool)?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;

    let mint = Mint::unpack_from_slice(&accounts.mint.data.borrow_mut())?;

    // Safety checks
    check_account_key(
        accounts.stake_pool,
        &stake_account.stake_pool,
        MediaError::WrongStakePool,
    )?;
    check_account_key(
        accounts.owner,
        &stake_account.owner,
        MediaError::StakeAccountOwnerMismatch,
    )?;
    check_account_key(
        accounts.central_vault,
        &central_state.central_vault,
        MediaError::WrongCentralVault,
    )?;
    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        MediaError::WrongMint,
    )?;

    let balances_and_inflation = calc_previous_balances_and_inflation(
        current_time,
        stake_account.last_claimed_time,
        &stake_pool,
    )?;

    let rewards = balances_and_inflation
        // Divide the accumulated total stake balance multiplied by the daily inflation
        .checked_div(mint.supply as u128)
        .ok_or(MediaError::Overflow)?
        // Multiply by % stakers receive
        .checked_mul(STAKER_MULTIPLIER as u128)
        .ok_or(MediaError::Overflow)?
        .checked_div(100)
        .ok_or(MediaError::Overflow)?
        // Multiply by the staker shares of the total pool
        .checked_mul(stake_account.stake_amount as u128)
        .ok_or(MediaError::Overflow)?
        .checked_div(stake_pool.header.total_staked as u128)
        .and_then(safe_downcast)
        .ok_or(MediaError::Overflow)?;

    // Transfer rewards
    let transfer_ix = transfer(
        &spl_token::ID,
        accounts.central_vault.key,
        accounts.rewards_destination.key,
        accounts.central_state.key,
        &[],
        rewards,
    )?;
    invoke_signed(
        &transfer_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.central_vault.clone(),
            accounts.central_state.clone(),
            accounts.rewards_destination.clone(),
        ],
        &[&[&program_id.to_bytes(), &[central_state.signer_nonce]]],
    )?;

    // Update states
    stake_account.last_claimed_time = current_time;
    stake_account.save(&mut accounts.stake_account.data.borrow_mut());

    Ok(())
}
