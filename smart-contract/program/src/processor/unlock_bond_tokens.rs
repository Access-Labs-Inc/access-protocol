//! Unlock ACCESS tokens bought through a bond account
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::error::AccessError;
use crate::state::{BondAccount, CentralState};
use bonfida_utils::{BorshSize, InstructionsAccount};

use crate::utils::{assert_bond_derivation, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The bond account
    #[cons(writable)]
    pub bond_account: &'a T,

    /// The account of the bond owner
    #[cons(writable, signer)]
    pub bond_owner: &'a T,

    /// The ACCESS mint token
    pub mint: &'a T,

    /// The ACCESS token destination
    #[cons(writable)]
    pub access_token_destination: &'a T,

    /// The account of the central state
    pub central_state: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            bond_account: next_account_info(accounts_iter)?,
            bond_owner: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            access_token_destination: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
        };

        // Check keys

        // Check ownership

        // Check signer
        check_signer(accounts.bond_owner, AccessError::BuyerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_unlock_bond_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;
    let central_state = CentralState::from_account_info(accounts.central_state)?;
    let mut bond = BondAccount::from_account_info(accounts.bond_account, false)?;
    let current_time = Clock::get()?.unix_timestamp;

    assert_bond_derivation(
        accounts.bond_account,
        accounts.bond_owner.key,
        bond.total_amount_sold,
        program_id,
    )?;

    if bond.total_amount_sold <= bond.total_unlocked_amount {
        msg!("All tokens have been unlocked");
        return Err(ProgramError::InvalidArgument);
    }

    if current_time < bond.unlock_start_date {
        msg!("The bond tokens have not started unlocking yet");
        return Err(ProgramError::InvalidArgument);
    }

    let delta = current_time
        .checked_sub(bond.last_claimed_time)
        .ok_or(AccessError::Overflow)?;

    if delta < bond.unlock_period {
        msg!("Need to wait the end of the current unlock period before unlocking the bond");
        return Err(ProgramError::InvalidArgument);
    }

    let missed_periods = delta
        .checked_div(bond.unlock_period)
        .ok_or(AccessError::Overflow)?;

    let unlock_amount = bond.calc_unlock_amount(missed_periods as u64)?;

    // Transfer tokens
    let transfer_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        accounts.mint.key,
        accounts.access_token_destination.key,
        accounts.central_state.key,
        &[],
        unlock_amount,
    )?;
    invoke_signed(
        &transfer_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.central_state.clone(),
            accounts.mint.clone(),
            accounts.access_token_destination.clone(),
        ],
        &[&[&program_id.to_bytes(), &[central_state.signer_nonce]]],
    )?;

    // Update bond state
    bond.last_unlock_time += missed_periods * bond.unlock_period;
    bond.total_unlocked_amount += unlock_amount;
    bond.total_staked -= unlock_amount;

    bond.save(&mut accounts.bond_account.data.borrow_mut());

    Ok(())
}
