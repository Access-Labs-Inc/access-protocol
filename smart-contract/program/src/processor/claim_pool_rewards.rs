//! Claim rewards of a stake pool
//! This instruction is used by stake pool owner for claiming their staking rewards
use crate::error::AccessError;
use crate::state::{CentralState, StakePool, Tag};
use crate::utils::{calc_reward_fp32, check_account_key, check_account_owner, check_signer};
use bonfida_utils::fp_math::safe_downcast;
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::invoke_signed;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::mint_to;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `claim_pool_rewards` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `claim_pool_rewards` instruction
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The stake pool owner account
    #[cons(signer)]
    pub owner: &'a T,

    /// The rewards destination
    #[cons(writable)]
    pub rewards_destination: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The mint address of the ACCESS token
    #[cons(writable)]
    pub mint: &'a T,

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
            owner: next_account_info(accounts_iter)?,
            rewards_destination: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.rewards_destination,
            &spl_token::ID,
            AccessError::WrongOwner,
        )?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.mint, &spl_token::ID, AccessError::WrongOwner)?;

        // Check signer
        check_signer(accounts.owner, AccessError::StakePoolOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_claim_pool_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let current_time = Clock::get()?.unix_timestamp;

    let central_state = CentralState::from_account_info(accounts.central_state)?;
    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;

    // Safety checks
    check_account_key(
        accounts.owner,
        &Pubkey::new(&stake_pool.header.owner),
        AccessError::WrongStakePoolAccountOwner,
    )?;
    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;

    let reward = safe_downcast(
        calc_reward_fp32(
            current_time,
            stake_pool.header.last_claimed_time,
            &stake_pool,
            false,
            false,
        )? >> 32,
    )
    .ok_or(AccessError::Overflow)?;

    msg!("Claiming pool rewards {}", reward);

    // Transfer rewards
    let transfer_ix = mint_to(
        &spl_token::ID,
        accounts.mint.key,
        accounts.rewards_destination.key,
        accounts.central_state.key,
        &[],
        reward,
    )?;
    invoke_signed(
        &transfer_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.central_state.clone(),
            accounts.mint.clone(),
            accounts.rewards_destination.clone(),
        ],
        &[&[&program_id.to_bytes(), &[central_state.signer_nonce]]],
    )?;

    // Update stake pool state
    stake_pool.header.last_claimed_time = current_time;

    Ok(())
}
