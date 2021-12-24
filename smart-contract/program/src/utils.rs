use crate::error::MediaError;
use crate::state::{BondAccount, AUTHORIZED_BOND_SELLERS};
use crate::state::{StakeAccount, StakePool, MEDIA_MINT, SECONDS_IN_DAY, STAKE_BUFFER_LEN};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

pub fn calc_previous_balances_and_inflation(
    current_time: i64,
    last_claimed_time: i64,
    stake_pool: &StakePool,
) -> Result<u128, ProgramError> {
    let last_full_day = current_time as u64 / SECONDS_IN_DAY;
    let mut last_claimed_day = last_claimed_time as u64 / SECONDS_IN_DAY;

    let mut i = last_full_day - last_claimed_day;
    i = (stake_pool.header.current_day_idx as u64 - i) % STAKE_BUFFER_LEN;

    let mut reward: u128 = 0;

    // Compute reward for all past days
    while last_claimed_day < last_full_day {
        reward = reward
            .checked_add(stake_pool.balances[i as usize])
            .ok_or(MediaError::Overflow)?;
        i = (i + 1) % STAKE_BUFFER_LEN;
        last_claimed_day += 1;
    }

    Ok(reward)
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
    if stake_pool.header.total_staked != 0 {
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
    if acc.mint != MEDIA_MINT {
        msg!("Invalid MEDIA mint");
        #[cfg(not(feature = "no-mint-check"))]
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn assert_uninitialized(account: &AccountInfo) -> ProgramResult {
    if !account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

pub fn assert_authorized_seller(seller: &AccountInfo, seller_index: usize) -> ProgramResult {
    let expected_seller = AUTHORIZED_BOND_SELLERS
        .get(seller_index)
        .ok_or(MediaError::UnauthorizedSeller)?;
    if seller.key != expected_seller {
        return Err(MediaError::UnauthorizedSeller.into());
    }
    Ok(())
}

pub fn assert_bond_derivation(
    account: &AccountInfo,
    owner: &Pubkey,
    total_amount_sold: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let (key, _nonce) = BondAccount::create_key(owner, total_amount_sold, program_id);
    check_account_key(account, &key, MediaError::AccountNotDeterministic)?;
    Ok(())
}

pub fn safe_downcast(n: u128) -> Option<u64> {
    static BOUND: u128 = u64::MAX as u128;
    if n > BOUND {
        None
    } else {
        Some(n as u64)
    }
}
