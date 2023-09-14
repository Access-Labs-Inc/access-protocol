//! Edit metadata
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    instruction::update_metadata_accounts_v2, pda::find_metadata_account, state::DataV2,
};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::AccessError};
use crate::instruction::ProgramInstruction::EditMetadata;
use crate::state::V1_INSTRUCTIONS_ALLOWED;
use crate::utils::{check_account_key, check_account_owner, check_signer};
use crate::state:: CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `edit_metadata` instruction
pub struct Params {
    // The name of the token
    pub name: String,
    // The symbol of the token
    pub symbol: String,
    // The URI of the token logo
    pub uri: String,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `change_inflation` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    pub central_state: &'a T,

    /// The account of the central state authority
    #[cons(signer)]
    pub authority: &'a T,

    /// The metadata account
    #[cons(writable)]
    pub metadata: &'a T,

    /// The metadata program account
    pub metadata_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
            metadata: next_account_info(accounts_iter)?,
            metadata_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.metadata_program,
            &mpl_token_metadata::ID,
            AccessError::WrongMplProgram,
        )?;

        // Check ownership
        check_account_owner(accounts.central_state, program_id, AccessError::WrongOwner)?;
        check_account_owner(
            accounts.metadata,
            &mpl_token_metadata::ID,
            AccessError::WrongOwner,
        )?;

        // Check signer
        check_signer(
            accounts.authority,
            AccessError::CentralStateAuthorityMustSign,
        )?;

        Ok(accounts)
    }
}

pub fn process_edit_metadata(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    if !V1_INSTRUCTIONS_ALLOWED {
        return Err(AccessError::DeprecatedInstruction.into());
    }

    let accounts = Accounts::parse(accounts, program_id)?;

    let central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(EditMetadata)?;
    let (metadata_key, _) = find_metadata_account(&central_state.token_mint);

    check_account_key(
        accounts.authority,
        &central_state.authority,
        AccessError::WrongCentralStateAuthority,
    )?;
    check_account_key(
        accounts.metadata,
        &metadata_key,
        AccessError::AccountNotDeterministic,
    )?;

    let data = DataV2 {
        name: params.name,
        uri: params.uri,
        symbol: params.symbol,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let ix = update_metadata_accounts_v2(
        *accounts.metadata_program.key,
        *accounts.metadata.key,
        *accounts.central_state.key,
        None,
        Some(data),
        None,
        Some(true),
    );
    invoke_signed(
        &ix,
        &[accounts.metadata.clone(), accounts.central_state.clone()],
        &[&[&program_id.to_bytes(), &[central_state.signer_nonce]]],
    )?;

    Ok(())
}
