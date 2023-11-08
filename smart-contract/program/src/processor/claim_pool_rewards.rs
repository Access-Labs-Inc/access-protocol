//! Claim rewards of a stake pool
//! This instruction is used by stake pool owner for claiming their staking rewards
use crate::error::AccessError;
use crate::state::{StakePool, Tag};
use crate::utils::{
    assert_no_close_or_delegate, calc_reward_fp32, check_account_key, check_account_owner,
    check_signer,
};
use bonfida_utils::fp_math::safe_downcast;
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::invoke_signed;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{instruction::mint_to, state::Account};
use crate::instruction::ProgramInstruction::ClaimPoolRewards;
use crate::state:: CentralStateV2;

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

    /// The owner's royalty split account to check if royalties need to be paid
    pub owner_royalty_account: &'a T,

    /// The optional royalty account to pay royalties to if there is no owner royalty split account
    /// this is be used by the Access NFT contract to pay royalties even for the NFTs owned by the owner
    pub royalty_account: Option<&'a T>,

    /// The royalty ATA account
    #[cons(writable)]
    pub royalty_ata: Option<&'a T>,
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
            owner_royalty_account: next_account_info(accounts_iter)?,
            royalty_account: next_account_info(accounts_iter).ok(),
            royalty_ata: next_account_info(accounts_iter).ok(),
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
        if let Some(royalty_account) = accounts.royalty_account {
            check_account_owner(
                royalty_account,
                program_id,
                AccessError::WrongRoyaltyAccountOwner,
            )?
        }
        if let Some(royalty_ata) = accounts.royalty_ata {
            check_account_owner(
                royalty_ata,
                &spl_token::ID,
                AccessError::WrongOwner,
            )?;
        }
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.mint, &spl_token::ID, AccessError::WrongOwner)?;

        Ok(accounts)
    }
}

pub fn process_claim_pool_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&ClaimPoolRewards)?;
    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;

    // Check that the royalty accounts are set up correctly
    let owner_must_pay = !accounts.owner_royalty_account.data_is_empty();
    if owner_must_pay && accounts.royalty_account.is_some() {
        return Err(AccessError::RoyaltyAccountMismatch.into());
    }

    let (derived_key, _) = RoyaltyAccount::create_key(&accounts.owner.key, program_id);
    check_account_key(
        accounts.owner_royalty_account,
        &derived_key,
        AccessError::AccountNotDeterministic,
    )?;

    // Check relationships between royalty accounts
    let mut royalty_account_data: Option<RoyaltyAccount> = None;
    if owner_must_pay {
        check_account_owner(
            accounts.owner_royalty_account,
            program_id,
            AccessError::WrongRoyaltyAccountOwner,
        )?;
        royalty_account_data = Some(RoyaltyAccount::from_account_info(accounts.owner_royalty_account)?);
        check_account_key(
            accounts.owner,
            &royalty_account_data.as_ref().unwrap().payer,
            AccessError::RoyaltyAccountMismatch,
        )?;
    } else if let Some(royalty_account) = accounts.royalty_account {
        check_signer(accounts.owner, AccessError::StakePoolOwnerMustSign)?;
        royalty_account_data = Some(RoyaltyAccount::from_account_info(royalty_account)?);
    }

    if let Some(royalty_account) = royalty_account_data.as_ref() {
        check_account_key(
            accounts.royalty_ata.ok_or(AccessError::RoyaltyAtaMismatch)?,
            &royalty_account.recipient_ata,
            AccessError::RoyaltyAtaMismatch,
        )?;
    } else {
        if accounts.royalty_ata.is_some() {
            return Err(AccessError::RoyaltyAtaMismatch.into());
        }
    }

    let destination_token_acc = Account::unpack(&accounts.rewards_destination.data.borrow())?;

    if destination_token_acc.mint != central_state.token_mint {
        return Err(AccessError::WrongMint.into());
    }

    msg!("Account owner: {}", destination_token_acc.owner);
    if destination_token_acc.owner.to_bytes() != stake_pool.header.owner {
        // If the destination does not belong to the stake pool owner he must sign
        check_signer(accounts.owner, AccessError::StakePoolOwnerMustSign)?;
    } else {
        assert_no_close_or_delegate(&destination_token_acc)?;
    }

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

    // Calculate the rewards (checks if the pool is cranked as well)
    let reward = calc_reward_fp32(
        central_state.last_snapshot_offset,
        stake_pool.header.last_claimed_offset,
        &stake_pool,
        false,
        false,
    )?;

    let reward = safe_downcast(((reward >> 31) + 1) >> 1).ok_or(AccessError::Overflow)?;

    // split the rewards if there is a royalty account
    let mut royalty_amount = 0;
    if let Some(royalty_account) = royalty_account_data {
        royalty_amount = royalty_account.calculate_royalty_amount(reward)?;
        reward.checked_sub(royalty_amount).ok_or(AccessError::Overflow)?;
    }

    msg!("Claiming pool rewards {}, royalties {}", reward, royalty_amount);

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
            accounts.central_state.clone(),
            accounts.mint.clone(),
            accounts.rewards_destination.clone(),
        ],
        &[&[&program_id.to_bytes(), &[central_state.bump_seed]]],
    )?;

    // Mint royalties
    if royalty_amount > 0 {
        let mint_royalty_ix = mint_to(
            &spl_token::ID,
            accounts.mint.key,
            accounts.royalty_ata.unwrap().key,
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
                accounts.royalty_account.unwrap().clone(),
            ],
            &[&[&program_id.to_bytes(), &[central_state.bump_seed]]],
        )?;
    }

    // Update stake pool state
    stake_pool.header.last_claimed_offset = central_state.last_snapshot_offset;

    Ok(())
}
