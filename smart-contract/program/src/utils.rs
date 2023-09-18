//! Utils
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

use crate::error::AccessError;
use crate::instruction::ProgramInstruction;
use crate::state::{AUTHORIZED_BOND_SELLERS, BondAccount};
use crate::state::{ACCESS_MINT, STAKE_BUFFER_LEN, StakeAccount, StakePoolRef};

/// Cumulate the claimable rewards from the last claimed day to the present.
/// Result is in FP32 format.
///
/// * `staker` Compute the reward for a staker or a pool owner
pub fn calc_reward_fp32(
    current_offset: u64,
    last_claimed_offset: u64,
    stake_pool: &StakePoolRef,
    staker: bool,
    allow_zero_rewards: bool,
) -> Result<u128, ProgramError> {
    let mut nb_days_to_claim = current_offset.saturating_sub(last_claimed_offset);
    msg!("Nb of days behind {}", nb_days_to_claim);
    msg!("Last claimed offset {}", last_claimed_offset);
    msg!("Current offset {}", current_offset);
    nb_days_to_claim = std::cmp::min(nb_days_to_claim, STAKE_BUFFER_LEN);
    msg!("Nb of days to claim {}", nb_days_to_claim);
    if nb_days_to_claim == 0 {
        if !allow_zero_rewards {
            msg!("No rewards to claim, no operation.");
            return Err(AccessError::NoOp.into());
        }
        return Ok(0);
    }

    if current_offset > stake_pool.header.current_day_idx as u64 {
        #[cfg(not(any(feature = "days-to-sec-10s", feature = "days-to-sec-15m")))]
        return Err(AccessError::PoolMustBeCranked.into());
    }

    msg!(
        "Stake pool current day idx wrapped {}",
        (stake_pool.header.current_day_idx as u64) % STAKE_BUFFER_LEN
    );
    // Saturating as we don't want to wrap around when there haven't been sufficient cranks
    let mut i = (stake_pool.header.current_day_idx as u64).saturating_sub(nb_days_to_claim)
        % STAKE_BUFFER_LEN;

    // Compute reward for all past days
    let mut reward: u128 = 0;
    loop {
        let curr_day_reward = if staker {
            stake_pool.balances[i as usize].stakers_reward
        } else {
            stake_pool.balances[i as usize].pool_reward
        };
        reward = reward
            .checked_add(curr_day_reward)
            .ok_or(AccessError::Overflow)?;
        i = (i + 1) % STAKE_BUFFER_LEN;
        if i == (stake_pool.header.current_day_idx as u64) % STAKE_BUFFER_LEN {
            break;
        }
    }

    msg!("Reward is {}", reward);

    if reward == 0 && !allow_zero_rewards {
        msg!("No rewards to claim, no operation.");
        return Err(AccessError::NoOp.into());
    }

    Ok(reward)
}

// returns a mask of the instructions that are supposed to be frozen
// if no instructions are provided, all instructions are frozen
#[allow(missing_docs)]
pub fn get_freeze_mask(instructions: Vec<ProgramInstruction>) -> u128 {
    if instructions.is_empty() {
        return 0;
    }
    let mut mask = u128::MAX;
    for instruction in instructions {
        let ix_mask = 1 << instruction as u32;
        mask &= !ix_mask;
    }
    mask
}

#[allow(missing_docs)]
pub fn get_unfreeze_mask(instructions: Vec<ProgramInstruction>) -> u128 {
    if instructions.is_empty() {
        return u128::MAX;
    }
    let mut mask = 0;
    for instruction in instructions {
        let ix_mask = 1 << instruction as u32;
        mask |= ix_mask;
    }
    mask
}

#[allow(missing_docs)]
pub fn check_account_key(account: &AccountInfo, key: &Pubkey, error: AccessError) -> ProgramResult {
    if account.key != key {
        return Err(error.into());
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn check_account_owner(
    account: &AccountInfo,
    owner: &Pubkey,
    error: AccessError,
) -> ProgramResult {
    if account.owner != owner {
        return Err(error.into());
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn check_signer(account: &AccountInfo, error: AccessError) -> ProgramResult {
    if !(account.is_signer) {
        return Err(error.into());
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn assert_empty_stake_pool(stake_pool: &StakePoolRef) -> ProgramResult {
    if stake_pool.header.total_staked != 0 {
        msg!("The stake pool must be empty");
        return Err(AccessError::StakePoolMustBeEmpty.into());
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn assert_empty_stake_account(stake_account: &StakeAccount) -> ProgramResult {
    if stake_account.stake_amount != 0 {
        msg!("The stake account must be empty");
        return Err(AccessError::StakeAccountMustBeEmpty.into());
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn assert_valid_vault(account: &AccountInfo, vault_signer: &Pubkey) -> ProgramResult {
    let acc = Account::unpack(&account.data.borrow())?;
    if &acc.owner != vault_signer {
        msg!("The vault account should be owned by the stake pool signer");
        return Err(ProgramError::InvalidArgument);
    }
    if acc.close_authority.is_some() || acc.delegate.is_some() {
        msg!("Invalid vault account provided");
        return Err(ProgramError::InvalidArgument);
    }
    if acc.mint != ACCESS_MINT {
        msg!("Invalid ACCESS mint");
        #[cfg(not(feature = "no-mint-check"))]
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn assert_uninitialized(account: &AccountInfo) -> ProgramResult {
    if !account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn assert_authorized_seller(seller: &AccountInfo, seller_index: usize) -> ProgramResult {
    let expected_seller = AUTHORIZED_BOND_SELLERS
        .get(seller_index)
        .ok_or(AccessError::UnauthorizedSeller)?;
    if seller.key != expected_seller {
        return Err(AccessError::UnauthorizedSeller.into());
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn assert_bond_derivation(
    account: &AccountInfo,
    owner: &Pubkey,
    total_amount_sold: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let (key, _nonce) = BondAccount::create_key(owner, total_amount_sold, program_id);
    check_account_key(account, &key, AccessError::AccountNotDeterministic)?;
    Ok(())
}

#[allow(missing_docs)]
pub fn assert_valid_fee(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    check_account_owner(account, &spl_token::ID, AccessError::WrongOwner)?;
    let acc = Account::unpack(&account.data.borrow())?;
    if owner != &acc.owner {
        msg!("Invalid fee account owner");
        return Err(ProgramError::IllegalOwner);
    }
    assert_no_close_or_delegate(&acc)?;
    Ok(())
}

#[allow(missing_docs)]
pub fn assert_no_close_or_delegate(token_account: &Account) -> ProgramResult {
    if token_account.delegate.is_some() || token_account.close_authority.is_some() {
        msg!("This token account cannot have a delegate or close authority");
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

#[allow(missing_docs)]
pub fn is_admin_instruction(instruction: &ProgramInstruction) -> bool {
    match instruction {
        ProgramInstruction::ChangeInflation |
        ProgramInstruction::AdminMint |
        ProgramInstruction::AdminFreeze |
        ProgramInstruction::ChangeCentralStateAuthority |
        ProgramInstruction::EditMetadata |
        ProgramInstruction::AdminSetupFeeSplit |
        ProgramInstruction::AdminSetProtocolFee |
        ProgramInstruction::AdminProgramFreeze
        => true,
        _ => false
    }
}