//! Allows central state authority to mint ACCESS tokens
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::AdminMint;
use crate::state::{CentralStateV2, V1_INSTRUCTIONS_ALLOWED};
use crate::utils::{check_account_key, check_account_owner, check_signer};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// The amount to be minted
    pub amount: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The central state authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The ACCESS mint token
    #[cons(writable)]
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
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            authority: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            access_token_destination: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;

        // Check signer
        check_signer(
            accounts.authority,
            AccessError::CentralStateAuthorityMustSign,
        )?;

        Ok(accounts)
    }
}

pub fn process_admin_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    if !V1_INSTRUCTIONS_ALLOWED {
        return Err(AccessError::DeprecatedInstruction.into());
    }

    let accounts = Accounts::parse(accounts, program_id)?;
    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(AdminMint)?;

    check_account_key(
        accounts.mint,
        &central_state.token_mint,
        AccessError::WrongMint,
    )?;
    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;

    // Transfer tokens
    let transfer_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        accounts.mint.key,
        accounts.access_token_destination.key,
        accounts.central_state.key,
        &[],
        params.amount,
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

    Ok(())
}
