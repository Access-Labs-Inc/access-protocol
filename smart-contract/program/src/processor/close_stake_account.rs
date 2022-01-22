//! Close a stake account
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::{
    assert_empty_stake_account, check_account_key, check_account_owner, check_signer,
};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::error::AccessError;
use crate::state::StakeAccount;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    // Stake pool
    pub stake_pool: Pubkey,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The owner of the stake account
    #[cons(writable, signer)]
    pub owner: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_account: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(accounts.stake_account, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(accounts.owner, AccessError::StakePoolOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_close_stake_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let Params { stake_pool } = params;

    // Derivation check doesn't seem necessary? If  stake_acc is program owned and has correct account tag and owner written in state?
    let (derived_stake_key, _) =
        StakeAccount::find_key(&accounts.owner.key, &stake_pool, program_id);

    check_account_key(
        accounts.stake_account,
        &derived_stake_key,
        AccessError::AccountNotDeterministic,
    )?;

    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account).unwrap();

    check_account_key(
        accounts.owner,
        &stake_account.owner,
        AccessError::WrongStakePoolOwner,
    )?;

    assert_empty_stake_account(&stake_account)?;

    stake_account.close();
    stake_account.save(&mut accounts.stake_account.data.borrow_mut());

    let mut stake_lamports = accounts.stake_account.lamports.borrow_mut();
    let mut owner_lamports = accounts.owner.lamports.borrow_mut();

    **owner_lamports += **stake_lamports;
    **stake_lamports = 0;

    Ok(())
}
