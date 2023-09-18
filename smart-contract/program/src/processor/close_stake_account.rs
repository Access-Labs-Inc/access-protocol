//! Close a stake account
//! This instruction can be used to close an empty stake account and collect the lamports
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
use bonfida_utils::BorshSize;
use bonfida_utils::InstructionsAccount;

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::CloseStakeAccount;
use crate::state::{StakeAccount, V1_INSTRUCTIONS_ALLOWED};
use crate::state:: CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `close_stake_account` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `close_stake_account` instruction
pub struct Accounts<'a, T> {
    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The owner of the stake account
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
            stake_account: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(accounts.stake_account, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(accounts.owner, AccessError::StakePoolOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_close_stake_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    if !V1_INSTRUCTIONS_ALLOWED {
        return Err(AccessError::DeprecatedInstruction.into());
    }

    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(CloseStakeAccount)?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;

    check_account_key(
        accounts.owner,
        &stake_account.owner,
        AccessError::WrongStakePoolOwner,
    )?;

    assert_empty_stake_account(&stake_account)?;

    stake_account.close();
    stake_account.save(&mut accounts.stake_account.data.borrow_mut())?;

    let mut stake_lamports = accounts.stake_account.lamports.borrow_mut();
    let mut owner_lamports = accounts.owner.lamports.borrow_mut();

    **owner_lamports += **stake_lamports;
    **stake_lamports = 0;

    Ok(())
}
