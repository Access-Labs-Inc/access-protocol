//! Claim bond rewards
//! This Instruction allows bond owners to claim their staking rewards
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

use crate::state::{BondAccount, CentralState, StakePool};
use crate::{error::AccessError, state::Tag};
use bonfida_utils::{fp_math::safe_downcast, BorshSize, InstructionsAccount};
use spl_token::{instruction::mint_to, state::Account};

use crate::utils::{
    assert_no_close_or_delegate, calc_reward_fp32, check_account_key, check_account_owner,
    check_signer,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `claim_bond_rewards` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `claim_bond_rewards` instruction
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The bond account
    #[cons(writable)]
    pub bond_account: &'a T,

    /// The bond account owner
    #[cons(signer)]
    pub bond_owner: &'a T,

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
            bond_account: next_account_info(accounts_iter)?,
            bond_owner: next_account_info(accounts_iter)?,
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
            accounts.bond_account,
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

        Ok(accounts)
    }
}

pub fn process_claim_bond_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    // If you don't want to issue rewards to bond holders
    if cfg!(feature = "claim-bond-rewards-off") {
        return Err(AccessError::NoOp.into());
    }

    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralState::from_account_info(accounts.central_state)?;
    let stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;
    let mut bond = BondAccount::from_account_info(accounts.bond_account, false)?;

    let destination_token_acc = Account::unpack(&accounts.rewards_destination.data.borrow())?;

    if destination_token_acc.owner != bond.owner {
        // If the destination does not belong to the bond owner he must sign
        check_signer(accounts.bond_owner, AccessError::BuyerMustSign)?;
    } else {
        assert_no_close_or_delegate(&destination_token_acc)?;
    }

    // Safety checks
    check_account_key(
        accounts.stake_pool,
        &bond.stake_pool,
        AccessError::WrongStakePool,
    )?;
    check_account_key(
        accounts.bond_owner,
        &bond.owner,
        AccessError::StakeAccountOwnerMismatch,
    )?;
    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;

    let reward = calc_reward_fp32(
        central_state.last_snapshot_offset,
        bond.last_claimed_offset,
        &stake_pool,
        true,
        false,
    )?
    // Multiply by the staker shares of the total pool
    .checked_mul(bond.total_staked as u128)
    .map(|r| ((r >> 31) + 1) >> 1)
    .and_then(safe_downcast)
    .ok_or(AccessError::Overflow)?;

    msg!("Claiming bond rewards {}", reward);
    msg!("Total staked {}", bond.total_staked);

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
            accounts.mint.clone(),
            accounts.central_state.clone(),
            accounts.rewards_destination.clone(),
        ],
        &[&[&program_id.to_bytes(), &[central_state.signer_nonce]]],
    )?;

    // Update states
    bond.last_claimed_offset = central_state.last_snapshot_offset;
    bond.save(&mut accounts.bond_account.data.borrow_mut())?;

    Ok(())
}
