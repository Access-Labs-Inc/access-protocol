//! Close a stake pool
//! This instruction can be used to close an empty stake pool and collect the lamports
use crate::{
    state::Tag,
    utils::{assert_empty_stake_pool, check_account_key, check_account_owner, check_signer},
};
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account;
use crate::state:: CentralStateV2;

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::CloseStakePool;
use crate::state::{StakePool, V1_INSTRUCTIONS_ALLOWED};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `close_stake_pool` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `close_stake_pool` instruction
pub struct Accounts<'a, T> {
    /// The account of the stake pool
    #[cons(writable)]
    pub stake_pool_account: &'a T,

    /// Pool vault
    pub pool_vault: &'a T,

    /// The owner of the stake pool
    #[cons(writable, signer)]
    pub owner: &'a T,

    /// The central state account
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
            pool_vault: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check keys

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(
            accounts.stake_pool_account,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(accounts.pool_vault, &spl_token::ID, AccessError::WrongOwner)?;

        // Check signer
        check_signer(accounts.owner, AccessError::StakePoolOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_close_stake_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    if !V1_INSTRUCTIONS_ALLOWED {
        return Err(AccessError::DeprecatedInstruction.into());
    }

    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&CloseStakePool)?;
    let mut stake_pool = StakePool::get_checked(
        accounts.stake_pool_account,
        vec![Tag::InactiveStakePool, Tag::StakePool],
    )?;

    check_account_key(
        accounts.owner,
        &Pubkey::from(stake_pool.header.owner),
        AccessError::WrongStakePoolOwner,
    )?;
    check_account_key(
        accounts.pool_vault,
        &Pubkey::from(stake_pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    let vault = Account::unpack_from_slice(&accounts.pool_vault.data.borrow_mut())?;

    if vault.amount != 0 {
        msg!("Vault isn't empty, there are remaining unstake requests");
        return Err(AccessError::PendingUnstakeRequests.into());
    }

    assert_empty_stake_pool(&stake_pool)?;

    stake_pool.header.close();

    let mut stake_pool_lamports = accounts.stake_pool_account.lamports.borrow_mut();
    let mut owner_lamports = accounts.owner.lamports.borrow_mut();

    **owner_lamports += **stake_pool_lamports;
    **stake_pool_lamports = 0;

    Ok(())
}
