use crate::error::AccessError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account;

pub fn check_account_key(account: &AccountInfo, key: &Pubkey, error: AccessError) -> ProgramResult {
    if account.key != key {
        return Err(error.into());
    }
    Ok(())
}

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

pub fn check_signer(account: &AccountInfo, error: AccessError) -> ProgramResult {
    if !(account.is_signer) {
        return Err(error.into());
    }
    Ok(())
}

pub fn assert_uninitialized(account: &AccountInfo) -> ProgramResult {
    if !account.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(())
}

pub fn assert_valid_fee(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    check_account_owner(account, &spl_token::ID, AccessError::WrongOwner)?;
    let acc = Account::unpack(&account.data.borrow())?;
    if &acc.owner != owner {
        msg!("Invalid fee account owner");
        return Err(ProgramError::IllegalOwner);
    }
    assert_no_close_or_delegate(&acc)?;
    Ok(())
}

pub fn assert_no_close_or_delegate(token_account: &Account) -> ProgramResult {
    if token_account.delegate.is_some() || token_account.close_authority.is_some() {
        msg!("This token account cannot have a delegate or close authority");
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}
