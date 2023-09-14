//! Create stake pool
use std::mem::size_of;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::{
    cpi::Cpi,
    error::AccessError,
    state::{RewardsTuple, StakePoolHeader, Tag, STAKE_BUFFER_LEN},
};
use crate::{state::StakePool, utils::assert_valid_vault};
use bonfida_utils::{BorshSize, InstructionsAccount};
use crate::instruction::ProgramInstruction::CreateStakePool;

use crate::utils::{check_account_key, check_account_owner};
use crate::state:: CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_stake_pool` instruction
pub struct Params {
    // Owner of the stake pool
    pub owner: Pubkey,
    // Minimum amount to stake
    pub minimum_stake_amount: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create_stake_pool` instruction
pub struct Accounts<'a, T> {
    /// The stake pool account
    #[cons(writable)]
    pub stake_pool_account: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The stake pool vault account
    pub vault: &'a T,

    /// The account of the central state
    pub central_state: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            stake_pool_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &system_program::ID,
            AccessError::WrongSystemProgram,
        )?;

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(
            accounts.stake_pool_account,
            &system_program::ID,
            AccessError::WrongOwner,
        )?;

        Ok(accounts)
    }
}

pub fn process_create_stake_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(CreateStakePool)?;

    let (derived_stake_key, nonce) = StakePool::find_key(&params.owner, program_id);

    check_account_key(
        accounts.stake_pool_account,
        &derived_stake_key,
        AccessError::AccountNotDeterministic,
    )?;

    assert_valid_vault(accounts.vault, &derived_stake_key)?;

    let stake_pool_header = StakePoolHeader::new(
        params.owner,
        nonce,
        *accounts.vault.key,
        params.minimum_stake_amount,
    )?;

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.stake_pool_account,
        &[StakePoolHeader::SEED, &params.owner.to_bytes(), &[nonce]],
        stake_pool_header.borsh_len() + size_of::<RewardsTuple>() * STAKE_BUFFER_LEN as usize,
    )?;

    let mut stake_pool =
        StakePool::get_checked(accounts.stake_pool_account, vec![Tag::Uninitialized])?;

    *stake_pool.header = stake_pool_header;

    Ok(())
}
