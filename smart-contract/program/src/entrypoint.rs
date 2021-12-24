use crate::error::MediaError;
use crate::processor::Processor;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, decode_error::DecodeError, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

/// The entrypoint to the program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<MediaError>();
        return Err(error);
    }
    Ok(())
}

impl PrintProgramError for MediaError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            MediaError::AlreadyInitialized => {
                msg!("Error: This account is already initialized")
            }
            MediaError::DataTypeMismatch => {
                msg!("Error: Data type mismatch")
            }
            MediaError::WrongSystemProgram => {
                msg!("Error: Wrong system program key");
            }
            MediaError::WrongRent => {
                msg!("Error: Wrong rent sysvar key")
            }
            MediaError::WrongOwner => {
                msg!("Error: Wrong account owner")
            }
            MediaError::AccountNotDeterministic => {
                msg!("Error: Account not generated deterministically")
            }
            MediaError::StakePoolOwnerMustSign => {
                msg!("Error: The stake pool owner must sign")
            }
            MediaError::WrongStakePoolOwner => {
                msg!("Error: The stake pool must be empty")
            }
            MediaError::StakePoolMustBeEmpty => {
                msg!("Error: The stake account must be empty")
            }
            MediaError::StakeAccountMustBeEmpty => {
                msg!("Error: The stake account owner must sign")
            }
            MediaError::StakeAccountOwnerMustSign => {
                msg!("Error: Wrong SPL token program ID")
            }
            MediaError::WrongSplTokenProgramId => {
                msg!("Error: Source token account must be owned to SPL Token")
            }
            MediaError::WrongTokenAccountOwner => {
                msg!("Error: Stake account must be program owned")
            }
            MediaError::WrongStakeAccountOwner => {
                msg!("Error: Stake pool account must be program owned")
            }
            MediaError::WrongStakePoolAccountOwner => {
                msg!("Error: Stake account owner mismatch")
            }
            MediaError::StakeAccountOwnerMismatch => {
                msg!("Error: Stake pool mismatch")
            }
            MediaError::StakePoolMismatch => {
                msg!("Error: Stake pool mismatch")
            }
            MediaError::StakePoolVaultMismatch => {
                msg!("Error: Stake pool vault mismatch")
            }
            MediaError::WrongCentralStateAuthority => {
                msg!("Error: Wrong central state authority")
            }
            MediaError::CentralStateAuthorityMustSign => {
                msg!("Error: The central state authority must sign")
            }
            MediaError::Overflow => {
                msg!("Error: Overflow")
            }
            MediaError::NoOp => {
                msg!("Error: operation is a no-op")
            }
            MediaError::WrongMint => {
                msg!("Error: Wrong mint")
            }
            MediaError::WrongCentralVault => {
                msg!("Error: Wrong central vault")
            }
            MediaError::WrongStakePool => {
                msg!("Error: Wrong stake pool")
            }
            MediaError::UnauthorizedSeller => {
                msg!("Error: Unauthorized bond seller")
            }
            MediaError::BondSellerMustSign => {
                msg!("Error: Bond seller must sign")
            }
            MediaError::BondSellerAlreadySigner => {
                msg!("Error: Bond seller has already signed")
            }
            MediaError::NotEnoughSellers => {
                msg!("Error: The bond does not have enough sellers")
            }
            MediaError::BuyerMustSign => {
                msg!("Error: The bond buyer must sign")
            }
        }
    }
}
