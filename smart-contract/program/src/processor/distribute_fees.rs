//! Close a stake pool
//! This instruction can be used to close an empty stake pool and collect the lamports
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program::invoke_signed,
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account;
use spl_token::instruction::transfer;

use crate::{
    state::{Tag, FeeSplit},
    utils::{assert_empty_stake_pool, check_account_key, check_account_owner, check_signer},
};
use crate::error::AccessError;
use crate::state::StakePool;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `close_stake_pool` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `close_stake_pool` instruction
pub struct Accounts<'a, T> {
    // todo comments
    #[cons(signer)]
    pub fee_payer: &'a T,

    #[cons(writable)]
    pub fee_split_pda: &'a T,

    #[cons(writable)]
    pub fee_split_ata: &'a T,

    pub spl_token_program: &'a T,
    /// Pool vault
    #[cons(writable)]
    pub token_accounts: &'a [T],
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            fee_payer: next_account_info(accounts_iter)?,
            fee_split_pda: next_account_info(accounts_iter)?,
            fee_split_ata: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            token_accounts: accounts_iter.as_slice(),
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.fee_split_pda,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(
            accounts.fee_split_ata,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        for token_account in accounts.token_accounts {
            check_account_owner(
                token_account,
                &spl_token::ID,
                AccessError::WrongTokenAccountOwner,
            )?;
        }

        // todo is this needed?
        // Check signer
        check_signer(accounts.fee_payer, AccessError::StakeAccountOwnerMustSign)?;
        Ok(accounts)
    }
}

pub fn process_distribute_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    // todo all checks!!!

    let fee_split = FeeSplit::from_account_info(accounts.fee_split_pda)?;

    let fee_split_ata = Account::unpack(&accounts.fee_split_ata.data.borrow())?;
    let total_balance = fee_split_ata.amount;
    msg!("Balance to distribute: {}", total_balance);

    if accounts.token_accounts.len() != fee_split.recipients.len() {
        msg!("Invalid count of the token accounts");
        return Err(AccessError::InvalidTokenAccount.into());
    }
    // todo check how many this can handle
    for (token_account, recipient) in accounts.token_accounts.iter().zip(fee_split.recipients.iter()) {
        if *token_account.key != recipient.ata {
            msg!("Invalid order of the token accounts");
            return Err(AccessError::InvalidTokenAccount.into());
        }
        let amount = total_balance * recipient.percentage / 100; // todo safe math
        let ix = spl_token::instruction::transfer(
            &spl_token::ID,
            accounts.fee_split_ata.key,
            &recipient.ata,
            accounts.fee_split_pda.key,
            &[],
            amount,
        ).unwrap();
        invoke_signed(
            &ix,
            &[
                accounts.spl_token_program.clone(),
                accounts.fee_split_ata.clone(),
                token_account.clone(),
                accounts.fee_split_pda.clone(),
            ],
            &[&[FeeSplit::SEED, &program_id.to_bytes(), &[fee_split.bump_seed]]],
        )?;
    }
    Ok(())
}
