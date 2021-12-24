//! Close a stake pool
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::{assert_empty_stake_pool, check_account_key, check_account_owner, check_signer};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::error::AccessError;
use crate::state::StakePool;

// TODO add admin close

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    // The PDA nonce
    pub nonce: u8,
    // Destination of the rewards
    pub destination: Pubkey,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The account of the stake pool
    #[cons(writable)]
    pub stake_pool_account: &'a T,

    /// The owner of the stake pool
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
            stake_pool_account: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
        };

        // Check keys

        // Check ownership
        check_account_owner(
            accounts.stake_pool_account,
            program_id,
            AccessError::WrongOwner,
        )?;

        // Check signer
        check_signer(accounts.owner, AccessError::StakePoolOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_close_stake_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let Params { nonce, destination } = params;

    let derived_stake_pool_key =
        StakePool::create_key(&nonce, accounts.owner.key, &destination, program_id);

    check_account_key(
        accounts.stake_pool_account,
        &derived_stake_pool_key,
        AccessError::AccountNotDeterministic,
    )?;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool_account).unwrap();

    check_account_key(
        accounts.owner,
        &Pubkey::new(&stake_pool.header.owner),
        AccessError::WrongStakePoolOwner,
    )?;

    assert_empty_stake_pool(&stake_pool)?;

    stake_pool.header.close();

    Ok(())
}
