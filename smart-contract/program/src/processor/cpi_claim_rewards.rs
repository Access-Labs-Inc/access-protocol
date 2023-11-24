//! Claim rewards of a stake account from the Access NFT Program
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use solana_program::program::invoke_signed;
use spl_token::{instruction::mint_to, state::Account};

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::ClaimRewards;
use crate::state::{ ACCESS_CNFT_PROGRAM_SIGNER, StakeAccount, StakePool, Tag};
use crate::state::CentralStateV2;
use crate::utils::{assert_no_close_or_delegate, calc_reward_fp32, check_account_key, check_account_owner, check_and_retrieve_royalty_account, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `claim_rewards` instruction
pub struct Params {
    // Should be false by default
    pub allow_zero_rewards: bool,
    /// cNFT Owner
    pub cnft_owner: Pubkey,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `claim_rewards` instruction
pub struct Accounts<'a, T> {
    ///  The central authority signer of the Access cNFT program
    #[cons(signer)]
    pub cnft_program_signer: &'a T,
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    // todo check that we are cheking this one level up
    /// The stake account - this is unchecked here as it is checked in the Access NFT program
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The rewards destination
    #[cons(writable)]
    pub rewards_destination: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The mint address of the ACS token
    #[cons(writable)]
    pub mint: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The owner's royalty split account to check if royalties need to be paid
    pub cnft_owner_royalty_account: &'a T,

    /// The royalty ATA account
    #[cons(writable)]
    pub royalty_ata: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            cnft_program_signer: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
            stake_account: next_account_info(accounts_iter)?,
            rewards_destination: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            cnft_owner_royalty_account: next_account_info(accounts_iter)?,
            royalty_ata: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;
        check_account_key(
            accounts.cnft_program_signer,
            &ACCESS_CNFT_PROGRAM_SIGNER,
            AccessError::WrongAccessCnftAuthority,
        )?;

        // Check ownership
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.stake_account,
            program_id,
            AccessError::WrongStakeAccountOwner,
        )?;
        check_account_owner(
            accounts.rewards_destination,
            &spl_token::ID,
            AccessError::WrongOwner,
        )?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.mint, &spl_token::ID, AccessError::WrongOwner)?;

        check_signer(accounts.cnft_program_signer, AccessError::AccessCnftAuthorityMustSign)?;
        Ok(accounts)
    }
}

pub fn process_cpi_claim_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&ClaimRewards)?;
    let stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;

    let royalty_account_data = check_and_retrieve_royalty_account(
        program_id,
        &params.cnft_owner,
        accounts.cnft_owner_royalty_account,
        Some(accounts.royalty_ata),
    )?;

    let destination_token_acc = Account::unpack(&accounts.rewards_destination.data.borrow())?;
    msg!("Account owner: {}", destination_token_acc.owner);

    if destination_token_acc.mint != central_state.token_mint {
        msg!("Invalid ACCESS mint");
        #[cfg(not(feature = "no-mint-check"))]
        return Err(AccessError::WrongMint.into());
    }

    check_account_key(
        accounts.stake_pool,
        &stake_account.stake_pool,
        AccessError::WrongStakePool,
    )?;
    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;

    // Calculate the rewards (checks if the pool is cranked as well)
    let mut  reward = calc_reward_fp32(
        central_state.last_snapshot_offset,
        stake_account.last_claimed_offset,
        &stake_pool,
        true,
        params.allow_zero_rewards,
    )?
        // Multiply by the staker shares of the total pool
        .checked_mul(stake_account.stake_amount as u128)
        .map(|r| ((r >> 31) + 1) >> 1)
        .ok_or(AccessError::Overflow)?
        .try_into()
        .map_err(|_|AccessError::Overflow)?;

    // split the rewards if there is a royalty account
    let mut royalty_amount = 0;
    if let Some(royalty_account) = royalty_account_data {
        royalty_amount = royalty_account.calculate_royalty_amount(reward)?;
        reward = reward.checked_sub(royalty_amount).ok_or(AccessError::Overflow)?;
    }

    msg!("Claiming rewards {}, royalties {}", reward, royalty_amount);

    // Mint rewards
    let mint_rewards_ix = mint_to(
        &spl_token::ID,
        accounts.mint.key,
        accounts.rewards_destination.key,
        accounts.central_state.key,
        &[],
        reward,
    )?;
    invoke_signed(
        &mint_rewards_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.mint.clone(),
            accounts.central_state.clone(),
            accounts.rewards_destination.clone(),
        ],
        &[&[&program_id.to_bytes(), &[central_state.bump_seed]]],
    )?;

    // Mint royalties
    if royalty_amount > 0 {
        let mint_royalty_ix = mint_to(
            &spl_token::ID,
            accounts.mint.key,
            accounts.royalty_ata.key,
            accounts.central_state.key,
            &[],
            royalty_amount,
        )?;
        invoke_signed(
            &mint_royalty_ix,
            &[
                accounts.spl_token_program.clone(),
                accounts.mint.clone(),
                accounts.central_state.clone(),
                accounts.royalty_ata.clone(),
            ],
            &[&[&program_id.to_bytes(), &[central_state.bump_seed]]],
        )?;
    }

    // Update states
    stake_account.last_claimed_offset = central_state.last_snapshot_offset;
    stake_account.save(&mut accounts.stake_account.data.borrow_mut())?;

    Ok(())
}
