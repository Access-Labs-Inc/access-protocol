//! Permissionless crank to update the stake pool rewards
//! This instructions updates the circular buffer with the pool balances multiplied by the current inflation

use bonfida_utils::{BorshSize, InstructionsAccount};
use bonfida_utils::fp_math::safe_downcast;
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
use crate::{
    error::AccessError,
    state::{RewardsTuple, StakePoolHeader, Tag},
};
use crate::state::{BondAccount, CentralState, StakePool};
use spl_associated_token_account::get_associated_token_address;
use spl_math::precise_number::PreciseNumber;
use spl_token::{instruction::mint_to, state::Account};

use crate::utils::{
    assert_no_close_or_delegate, calc_reward_fp32_v2, check_account_key, check_account_owner,
    check_signer,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `crank` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `crank` instruction
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,
    /// The stake pool owner account
    #[cons(writable)]
    pub owner: &'a T,
    /// The rewards destination for the pool owner
    #[cons(writable)]
    pub rewards_destination: &'a T,
    /// The account of the central state
    #[cons(writable)]
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

        Ok(accounts)
    }
}

pub fn process_crank(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    msg!("Processing crank instruction");
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut central_state = CentralState::from_account_info(accounts.central_state)?;
    // todo v2 pools
    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool, Tag::StakePoolV2])?;

    let destination_token_acc = Account::unpack(&accounts.rewards_destination.data.borrow())?;

    if destination_token_acc.mint != central_state.token_mint {
        return Err(AccessError::WrongMint.into());
    }

    // check if the destination belongs to the pool owner
    if destination_token_acc.owner.to_bytes() != stake_pool.header.owner {
        return Err(AccessError::WrongDestinationAccount.into());
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

    // check if we need to do a system wide snapshot
    let current_offset = central_state.get_current_offset()?;
    if central_state.last_snapshot_offset < current_offset {
        central_state.total_staked_snapshot = central_state.total_staked;
        central_state.last_snapshot_offset = current_offset;
        central_state.save(&mut accounts.central_state.data.borrow_mut())?;
    }

    // check that the pool is not already cranked
    if stake_pool.header.current_day_idx as u64 == central_state.last_snapshot_offset {
        return Err(AccessError::NoOp.into());
    }

    msg!("Total staked in pool {}", stake_pool.header.total_staked);
    msg!("Daily inflation {}", central_state.daily_inflation);
    msg!("Total staked {}", central_state.total_staked);
    msg!(
        "Total staked snapshot {}",
        central_state.total_staked_snapshot
    );

    // get the pool staked amount at the time of last system snapshot
    let total_staked_snapshot = stake_pool.header.total_staked as u128;

    let mut supporter_reward_base = 0;
    if total_staked_snapshot != 0 {
        // stakers_reward = [(pool_total_staked << 32) * inflation * stakers_part] / (100 * total_staked * pool_total_staked)
        supporter_reward_base = ((central_state.daily_inflation as u128) << 32)
            .checked_mul(stake_pool.header.stakers_part as u128)
            .ok_or(AccessError::Overflow)?
            .checked_div(100u128)
            .ok_or(AccessError::Overflow)?
            .checked_div(central_state.total_staked_snapshot as u128)
            .unwrap_or(0);
    };

    msg!("Stakers reward {}", supporter_reward_base);

    let precise_total_staked_snapshot = PreciseNumber::new(total_staked_snapshot.checked_shl(32)
        .ok_or(AccessError::Overflow)?)
        .ok_or(AccessError::Overflow)?;
    let precise_daily_inflation = PreciseNumber::new(central_state.daily_inflation as u128)
        .ok_or(AccessError::Overflow)?;
    let precise_system_staked_snapshot = PreciseNumber::new(central_state.total_staked_snapshot as u128)
        .ok_or(AccessError::Overflow)?;

    // pool_rewards = [(pool_total_staked << 32) * inflation * (100 - stakers_part)] / (100 * total_staked)
    let precise_pool_reward = (precise_total_staked_snapshot)
        .checked_mul(&precise_daily_inflation)
        .ok_or(AccessError::Overflow)?
        .checked_mul(
            &PreciseNumber::new(100u64
                                    .checked_sub(stake_pool.header.stakers_part)
                                    .ok_or(AccessError::Overflow)? as u128,
            ).ok_or(AccessError::Overflow)?,
        )
        .ok_or(AccessError::Overflow)?
        .checked_div(&PreciseNumber::new(100u128).ok_or(AccessError::Overflow)?)
        .ok_or(AccessError::Overflow)?
        .checked_div(&precise_system_staked_snapshot)
        .unwrap_or(PreciseNumber::new(0).ok_or(AccessError::Overflow)?);

    let pool_reward = precise_pool_reward.to_imprecise().ok_or(AccessError::Overflow)?;

    msg!("Pool reward {}", pool_reward);

    let total_claimable_rewards = (((pool_reward >> 31) + 1) >> 1)
        .checked_add(
            ((supporter_reward_base.checked_mul(total_staked_snapshot)
                .ok_or(AccessError::Overflow)? >> 31) + 1) >> 1
        ).ok_or(AccessError::Overflow)?;

    msg!("Total claimable rewards {}", total_claimable_rewards);

    assert!(total_claimable_rewards <= (central_state.daily_inflation as u128)
        .checked_add(1_000_000).ok_or(AccessError::Overflow)?);

    // Mint pool rewards directly to the pool owner
    let mint_ix = mint_to(
        &spl_token::ID,
        accounts.mint.key,
        accounts.rewards_destination.key,
        accounts.central_state.key,
        &[],
        safe_downcast(((pool_reward >> 31) + 1) >> 1).ok_or(AccessError::Overflow)?,
    )?;
    invoke_signed(
        &mint_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.central_state.clone(),
            accounts.mint.clone(),
            accounts.rewards_destination.clone(),
        ],
        &[&[&program_id.to_bytes(), &[central_state.signer_nonce]]],
    )?;

    // todo resolve v2 - if we want to keep v1 as well or not
    if stake_pool.header.tag == Tag::StakePool as u8 {
        stake_pool.push_balances_buff(
            current_offset,
            RewardsTuple {
                pool_reward: 0,
                stakers_reward: supporter_reward_base,
            },
        )?;
    } else if stake_pool.header.tag == Tag::StakePoolV2 as u8{
        let mut stake_pool_v2 = StakePool::get_checked_v2(accounts.stake_pool, vec![Tag::StakePoolV2])?;
        stake_pool_v2.push_balances_buff_v2(
            current_offset,
            supporter_reward_base,
        )?;
    } else {
        return Err(AccessError::NoOp.into());
    }
    Ok(())
}
