//! Claim rewards of a bond V2  from the Access NFT Program
use crate::error::AccessError;
use crate::state::BondV2Account;
use crate::state::{StakePool, RoyaltyAccount, Tag,ACCESS_NFT_PROGRAM_SIGNER};
use crate::utils::{
    calc_reward_fp32, check_account_key, check_account_owner,
    check_signer, retrieve_royalty_account
};
use std::convert::TryInto;
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
use crate::instruction::ProgramInstruction::ClaimBondV2Rewards;
use crate::state:: CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `claim_bond_v2_rewards` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `claim_bond_v2_rewards` instruction
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub pool: &'a T,

    /// The Bond V2 account
    #[cons(writable)]
    pub bond_v2_account: &'a T,

    /// The owner of the Bond V2 account
    #[cons(signer)]
    pub owner: &'a T,

    /// The rewards destination
    #[cons(writable)]
    pub rewards_destination: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The mint address of the ACS token
    #[cons(writable)]
    pub mint: &'a T,

    /// The Access NFT program signer - to handle different royalty account
    pub access_nft_signer: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The owner's royalty split account to check if royalties need to be paid
    pub owner_royalty_account: &'a T,

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
            pool: next_account_info(accounts_iter)?,
            bond_v2_account: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            rewards_destination: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            access_nft_signer: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            owner_royalty_account: next_account_info(accounts_iter)?,
            royalty_ata: next_account_info(accounts_iter).ok(),
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;
        check_account_key(
            accounts.access_nft_signer,
            &ACCESS_NFT_PROGRAM_SIGNER,
            AccessError::WrongAccessCnftAuthority,
        )?;

        // Check ownership
        check_account_owner(
            accounts.pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.bond_v2_account,
            program_id,
            AccessError::WrongBondAccountOwner,
        )?;
        check_account_owner(
            accounts.rewards_destination,
            &spl_token::ID,
            AccessError::WrongOwner,
        )?;
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.mint, &spl_token::ID, AccessError::WrongOwner)?;

        // Check signers
        check_signer(accounts.owner, AccessError::StakeAccountOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_claim_bond_v2_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&ClaimBondV2Rewards)?;
    let stake_pool = StakePool::get_checked(accounts.pool, vec![Tag::StakePool])?;
    let mut bond_v2_account = BondV2Account::from_account_info(accounts.bond_v2_account)?;

    let destination_token_acc = Account::unpack(&accounts.rewards_destination.data.borrow())?;

    if destination_token_acc.mint != central_state.token_mint {
        msg!("Invalid ACCESS mint");
        #[cfg(not(feature = "no-mint-check"))]
        return Err(AccessError::WrongMint.into());
    }

    msg!("Token account owner: {}", destination_token_acc.owner);

    // We only allow custom royalty account and unchecked destination account when this is a CPI call from the NFT program.
    if !accounts.access_nft_signer.is_signer {
        let (derived_key, _) = RoyaltyAccount::create_key(accounts.owner.key, program_id);
        check_account_key(
            accounts.owner_royalty_account,
            &derived_key,
            AccessError::AccountNotDeterministic,
        )?;
    }

    let royalty_account_data = retrieve_royalty_account(
        accounts.owner_royalty_account,
        accounts.royalty_ata,
    )?;

    check_account_key(
        accounts.pool,
        &bond_v2_account.pool,
        AccessError::WrongStakePool,
    )?;
    check_account_key(
        accounts.owner,
        &bond_v2_account.owner,
        AccessError::WrongOwner,
    )?;
    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;

    // Calculate the rewards (checks if the pool is cranked as well)
    let mut reward = calc_reward_fp32(
        central_state.last_snapshot_offset,
        bond_v2_account.last_claimed_offset,
        &stake_pool,
        true,
        false,
    )?
    // Multiply by the staker shares of the total pool
    .checked_mul(bond_v2_account.amount as u128)
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
                accounts.royalty_ata.unwrap().clone(),
            ],
            &[&[&program_id.to_bytes(), &[central_state.bump_seed]]],
        )?;
    }

    // Update states
    bond_v2_account.last_claimed_offset = central_state.last_snapshot_offset;
    bond_v2_account.save(&mut accounts.bond_v2_account.data.borrow_mut())?;

    Ok(())
}
