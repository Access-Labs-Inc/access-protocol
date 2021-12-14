use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::utils::{
    assert_empty_stake_account, check_account_key, check_account_owner, check_signer,
};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::error::MediaError;
use crate::state::StakeAccount;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    // The PDA nonce
    pub nonce: u8,
    // Owner of the stake account
    pub owner: Pubkey,
    // Stake pool
    pub stake_pool: Pubkey,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    #[cons(writable)]
    stake_account: &'a T,
    #[cons(writable, signer)]
    owner: &'a T,
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
        check_account_owner(accounts.stake_account, program_id, MediaError::WrongOwner)?;

        // Check signer
        check_signer(accounts.owner, MediaError::StakePoolOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_close_stake_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let Params {
        nonce,
        owner,
        stake_pool,
    } = params;

    let derived_stake_key = StakeAccount::create_key(&nonce, &owner, &stake_pool, program_id);

    check_account_key(
        accounts.stake_account,
        &derived_stake_key,
        MediaError::AccountNotDeterministic,
    )?;

    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account).unwrap();

    check_account_key(
        accounts.owner,
        &stake_account.owner,
        MediaError::WrongStakePoolOwner,
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
