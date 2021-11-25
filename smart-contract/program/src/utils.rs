use crate::state::{CentralState, StakeAccount, StakePool, MEDIA_MINT, SECONDS_IN_DAY};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::{Account, Mint};

use crate::error::MediaError;

pub fn get_balance(account: &Account) -> u64 {
    account.amount
}

pub fn get_supply(mint: &Mint) -> u64 {
    mint.supply
}

pub fn calc_reward(
    current_time: u64,
    stake_pool: &StakePool,
    central_state: &CentralState,
    mint: &Mint,
) -> u64 {
    let period = current_time
        .checked_sub(stake_pool.last_crank_time as u64)
        .unwrap()
        .checked_div(SECONDS_IN_DAY)
        .unwrap();

    let amount = stake_pool
        .total_staked
        .checked_mul(central_state.daily_inflation)
        .unwrap()
        .checked_div(mint.supply)
        .unwrap();

    amount.checked_mul(period).unwrap()
}

pub fn check_account_key(account: &AccountInfo, key: &Pubkey, error: MediaError) -> ProgramResult {
    if account.key != key {
        return Err(error.into());
    }
    Ok(())
}

pub fn check_account_owner(
    account: &AccountInfo,
    owner: &Pubkey,
    error: MediaError,
) -> ProgramResult {
    if account.owner != owner {
        return Err(error.into());
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo, error: MediaError) -> ProgramResult {
    if !(account.is_signer) {
        return Err(error.into());
    }
    Ok(())
}

pub fn assert_empty_stake_pool(stake_pool: &StakePool) -> ProgramResult {
    if stake_pool.total_staked != 0 {
        msg!("The stake pool must be empty");
        return Err(MediaError::StakePoolMustBeEmpty.into());
    }
    Ok(())
}

pub fn assert_empty_stake_account(stake_account: &StakeAccount) -> ProgramResult {
    if stake_account.stake_amount != 0 {
        msg!("The stake account must be empty");
        return Err(MediaError::StakeAccountMustBeEmpty.into());
    }
    Ok(())
}

pub fn check_vault_account_and_get_mint(
    account: &AccountInfo,
    stake_pool_signer: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    let acc = Account::unpack(&account.data.borrow())?;
    if &acc.owner != stake_pool_signer {
        msg!("The vault account should be owned by the stake pool signer");
        return Err(ProgramError::InvalidArgument);
    }
    if acc.close_authority.is_some() || acc.delegate.is_some() {
        msg!("Invalid vault account provided");
        return Err(ProgramError::InvalidArgument);
    }
    Ok(acc.mint)
}

pub fn assert_valid_vault(account: &AccountInfo, stake_pool_signer: &Pubkey) -> ProgramResult {
    let mint = check_vault_account_and_get_mint(account, stake_pool_signer)?;
    if mint != MEDIA_MINT {
        msg!("Invalid MEDIA mint");
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}
