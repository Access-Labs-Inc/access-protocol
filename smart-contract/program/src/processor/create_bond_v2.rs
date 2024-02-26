//! Create a Bond V2
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
    sysvar::Sysvar,
};




use crate::{cpi::Cpi, state::Tag};
use crate::error::AccessError;
use crate::instruction::ProgramInstruction::CreateBondV2;
use crate::state::{BondV2Account, StakePool};
use crate::state::CentralStateV2;
use crate::utils::{
    check_account_key, check_account_owner,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `create_bond_v2` instruction
pub struct Params {
    /// The timestamp of the unlock, if any
    pub unlock_timestamp: Option<i64>,
    ///  Owner of the bond account
    pub owner: Pubkey,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create_bond_v2` instruction
pub struct Accounts<'a, T> {
    /// The bond account
    #[cons(writable)]
    pub bond_v2_account: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The pool account
    pub pool: &'a T,

    /// The fee account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// Central state
    pub central_state: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            bond_v2_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            pool: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
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
            accounts.bond_v2_account,
            &system_program::ID,
            AccessError::WrongOwner,
        )?;
        check_account_owner(
            accounts.pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;


        Ok(accounts)
    }
}

pub fn process_create_bond_v2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params {
        unlock_timestamp,
        owner
    } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&CreateBondV2)?;
    let pool = StakePool::get_checked(accounts.pool, vec![Tag::StakePool])?;

    let (derived_key, bump_seed) =
        BondV2Account::create_key(&owner, accounts.pool.key, unlock_timestamp, program_id);

    check_account_key(
        accounts.bond_v2_account,
        &derived_key,
        AccessError::AccountNotDeterministic,
    )?;

    let current_time = Clock::get()?.unix_timestamp;
    if unlock_timestamp.is_some() && current_time > unlock_timestamp.unwrap() {
        msg!("Cannot create a bond with an unlock timestamp in the past");
        return Err(ProgramError::InvalidArgument);
    }

    let bond = BondV2Account::new(
        owner,
        *accounts.pool.key,
        pool.header.minimum_stake_amount,
        unlock_timestamp,
    );

    // Create bond account
    let seeds: &[&[u8]] = &[
        BondV2Account::SEED,
        &owner.to_bytes(),
        &accounts.pool.key.to_bytes(),
        &unlock_timestamp.unwrap_or(0).to_le_bytes(),
        &[bump_seed],
    ];

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.bond_v2_account,
        seeds,
        bond.borsh_len(),
    )?;

    bond.save(&mut accounts.bond_v2_account.data.borrow_mut())?;
    Ok(())
}
