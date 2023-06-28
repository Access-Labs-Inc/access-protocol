//! Mint subscription
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{AccountInfo, next_account_info}, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey, system_instruction, system_program};
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_pack::Pack;
use spl_token::instruction::transfer;

use {
    solana_program::{
        entrypoint,
    },
    spl_token::{
        instruction as token_instruction,
    },
    spl_associated_token_account::{
        instruction as token_account_instruction,
    },
};

use crate::{
    utils::{check_account_key, check_account_owner, check_signer},
};
use crate::error::AccessError;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `mint_subscription` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `mint_subscription` instruction
pub struct Accounts<'a, T> {
    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The nft mint
    #[cons(signer, writable)]
    pub mint: &'a T,

    /// The nft token account
    #[cons(writable)]
    pub token_account: &'a T,

    /// The mint authority
    #[cons(signer)]
    pub mint_authority: &'a T,

    /// The rent sysvar
    pub rent: &'a T,

    /// The system program
    pub system_program: &'a T,

    /// The SPL token program
    pub token_program: &'a T,

    /// The associated token program
    pub associated_token_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            fee_payer: next_account_info(accounts_iter)?,
            mint: next_account_info(accounts_iter)?,
            token_account: next_account_info(accounts_iter)?,
            mint_authority: next_account_info(accounts_iter)?,
            rent: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            token_program: next_account_info(accounts_iter)?,
            associated_token_program: next_account_info(accounts_iter)?,
        };

        // // Check keys
        // check_account_key(
        //     accounts.system_program,
        //     &system_program::ID,
        //     AccessError::WrongSystemProgram,
        // )?;
        //
        // // Check ownership - mint must be a PDA
        // check_account_owner(
        //     accounts.mint,
        //     &system_program::ID,
        //     AccessError::WrongOwner,
        // )?;

        Ok(accounts)
    }
}

pub fn process_mint_subscription(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    msg!("Processing mint_subscription instruction...");
    let accounts = Accounts::parse(accounts, program_id)?;

    // todo put this elsewhere - probably state.rs
    // let seeds: &[&[u8]] = &[
    //     b"subscription_nft", // todo into a constant
    //     &owner.to_bytes(),
    //     &stake_pool.to_bytes(),
    //     &[*nonce],
    // ];
    // let mint = Pubkey::create_program_address(seeds, program_id).map_err(|_| ProgramError::InvalidSeeds);
    // todo check if mint exists

    msg!("Creating mint account...");
    msg!("Mint: {}", accounts.mint.key);
    invoke(
        &system_instruction::create_account(
            &accounts.fee_payer.key,
            &accounts.mint.key,
            LAMPORTS_PER_SOL,
            82,
            &accounts.token_program.key,
        ),
        &[
            accounts.mint.clone(),
            accounts.fee_payer.clone(),
            accounts.token_program.clone(),
        ]
    )?;

    msg!("Initializing mint account...");
    msg!("Mint: {}", accounts.mint.key);
    invoke(
        &token_instruction::initialize_mint(
            &accounts.token_program.key,
            &accounts.mint.key,
            &accounts.mint_authority.key,
            Some(&accounts.mint_authority.key),
            0,
        )?,
        &[
            accounts.mint.clone(),
            accounts.mint_authority.clone(),
            accounts.token_program.clone(),
            accounts.rent.clone(),
        ]
    )?;

    msg!("Creating token account...");
    msg!("Token Address: {}", accounts.token_account.key);
    invoke(
        &token_account_instruction::create_associated_token_account(
            &accounts.fee_payer.key,
            &accounts.mint_authority.key,
            &accounts.mint.key,
            &accounts.token_program.key,
        ),
        &[
            accounts.fee_payer.clone(),
            accounts.mint.clone(),
            accounts.token_account.clone(),
            accounts.mint_authority.clone(),
            accounts.token_program.clone(),
            accounts.associated_token_program.clone(),
        ]
    )?;

    msg!("Minting token to token account...");
    msg!("Mint: {}", accounts.mint.key);
    msg!("Token Address: {}", accounts.token_account.key);
    invoke(
        &token_instruction::mint_to(
            &accounts.token_program.key,
            &accounts.mint.key,
            &accounts.token_account.key,
            &accounts.mint_authority.key,
            &[&accounts.mint_authority.key],
            1,
        )?,
        &[
            accounts.mint.clone(),
            accounts.mint_authority.clone(),
            accounts.token_account.clone(),
            accounts.token_program.clone(),
            accounts.rent.clone(),
        ]
    )?;

    msg!("Token mint process completed successfully.");
    Ok(())
}
