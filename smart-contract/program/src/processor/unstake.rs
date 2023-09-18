//! Unstake
use crate::{
    state::{Tag},
    utils::{check_account_key, check_account_owner, check_signer},
};
use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::program::invoke_signed;
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::transfer;
use spl_token::state::Account;

use crate::error::AccessError;
use crate::instruction::ProgramInstruction::Unstake;
use crate::state::{BondAccount, StakeAccount, StakePool, StakePoolHeader};
use crate::state:: CentralStateV2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
/// The required parameters for the `unstake` instruction
pub struct Params {
    // Amount to unstake
    pub amount: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `unstake` instruction
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The stake account
    #[cons(writable)]
    pub stake_account: &'a T,

    /// The stake pool account
    #[cons(writable)]
    pub stake_pool: &'a T,

    /// The owner of the stake account
    #[cons(signer)]
    pub owner: &'a T,

    /// The destination of the staked tokens
    #[cons(writable)]
    pub destination_token: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The stake pool vault
    #[cons(writable)]
    pub vault: &'a T,

    /// Optional bond account to be able to stake under the minimum
    pub bond_account: Option<&'a T>,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state: next_account_info(accounts_iter)?,
            stake_account: next_account_info(accounts_iter)?,
            stake_pool: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            destination_token: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            bond_account: next_account_info(accounts_iter).ok(),
        };

        // Check keys
        check_account_key(
            accounts.spl_token_program,
            &spl_token::ID,
            AccessError::WrongSplTokenProgramId,
        )?;

        // Check ownership
        check_account_owner(
            accounts.central_state,
            program_id,
            AccessError::WrongOwner,
        )?;
        check_account_owner(
            accounts.stake_account,
            program_id,
            AccessError::WrongStakeAccountOwner,
        )?;
        check_account_owner(
            accounts.stake_pool,
            program_id,
            AccessError::WrongStakePoolAccountOwner,
        )?;
        check_account_owner(
            accounts.destination_token,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        check_account_owner(
            accounts.vault,
            &spl_token::ID,
            AccessError::WrongTokenAccountOwner,
        )?;
        if let Some(bond_account) = accounts.bond_account {
            check_account_owner(bond_account, program_id, AccessError::WrongBondAccountOwner)?
        }

        // Check signer
        check_signer(accounts.owner, AccessError::StakeAccountOwnerMustSign)?;

        Ok(accounts)
    }
}

pub fn process_unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let Params { amount } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut stake_pool = StakePool::get_checked(accounts.stake_pool, vec![Tag::StakePool])?;
    let mut stake_account = StakeAccount::from_account_info(accounts.stake_account)?;
    let mut central_state = CentralStateV2::from_account_info(accounts.central_state)?;
    central_state.assert_instruction_allowed(&Unstake)?;

    let destination_token_acc = Account::unpack(&accounts.destination_token.data.borrow())?;
    if destination_token_acc.mint != central_state.token_mint {
        return Err(AccessError::WrongMint.into());
    }

    if stake_account.last_claimed_offset < stake_pool.header.current_day_idx as u64 {
        return Err(AccessError::UnclaimedRewards.into());
    }
    if (stake_pool.header.current_day_idx as u64) < central_state.get_current_offset()? {
        return Err(AccessError::PoolMustBeCranked.into());
    }

    check_account_key(
        accounts.owner,
        &stake_account.owner,
        AccessError::StakeAccountOwnerMismatch,
    )?;
    check_account_key(
        accounts.stake_pool,
        &stake_account.stake_pool,
        AccessError::StakePoolMismatch,
    )?;
    check_account_key(
        accounts.vault,
        &Pubkey::new(&stake_pool.header.vault),
        AccessError::StakePoolVaultMismatch,
    )?;

    let mut amount_in_bonds: u64 = 0;
    if let Some(bond_account) = accounts.bond_account {
        let bond_account = BondAccount::from_account_info(bond_account, false)?;
        check_account_key(accounts.owner, &bond_account.owner, AccessError::WrongOwner)?;
        check_account_key(
            accounts.stake_pool,
            &bond_account.stake_pool,
            AccessError::StakePoolMismatch,
        )?;

        amount_in_bonds = bond_account.total_staked;
    }

    if stake_pool.header.minimum_stake_amount < stake_account.pool_minimum_at_creation {
        stake_account.pool_minimum_at_creation = stake_pool.header.minimum_stake_amount
    }

    // Can unstake either above the minimum or everything - includes the bond account
    let new_total_in_pool = stake_account
        .stake_amount
        .checked_add(amount_in_bonds)
        .ok_or(AccessError::Overflow)?
        .checked_sub(amount)
        .ok_or(AccessError::Overflow)?;
    if stake_account.stake_amount != amount
        && new_total_in_pool < stake_account.pool_minimum_at_creation
    {
        return Err(AccessError::InvalidUnstakeAmount.into());
    }

    // Update stake account
    stake_account.withdraw(amount)?;
    stake_pool.header.withdraw(amount)?;

    // Transfer tokens
    let signer_seeds: &[&[u8]] = &[
        StakePoolHeader::SEED,
        &stake_pool.header.owner.clone(),
        &[stake_pool.header.nonce],
    ];
    let transfer_instruction = transfer(
        &spl_token::ID,
        accounts.vault.key,
        accounts.destination_token.key,
        accounts.stake_pool.key,
        &[],
        amount,
    )?;

    drop(stake_pool);

    invoke_signed(
        &transfer_instruction,
        &[
            accounts.spl_token_program.clone(),
            accounts.vault.clone(),
            accounts.destination_token.clone(),
            accounts.stake_pool.clone(),
        ],
        &[signer_seeds],
    )?;

    // Save states
    stake_account.save(&mut accounts.stake_account.data.borrow_mut())?;

    //Update central state
    central_state.total_staked = central_state
        .total_staked
        .checked_sub(amount)
        .ok_or(AccessError::Overflow)?;
    central_state.save(&mut accounts.central_state.data.borrow_mut())?;

    Ok(())
}
