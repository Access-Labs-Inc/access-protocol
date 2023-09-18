use crate::error::AccessError;
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
        error.print::<AccessError>();
        return Err(error);
    }
    Ok(())
}

impl PrintProgramError for AccessError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            AccessError::AlreadyInitialized => {
                msg!("Error: This account is already initialized")
            }
            AccessError::DataTypeMismatch => {
                msg!("Error: Data type mismatch")
            }
            AccessError::WrongSystemProgram => {
                msg!("Error: Wrong system program key");
            }
            AccessError::WrongRent => {
                msg!("Error: Wrong rent sysvar key")
            }
            AccessError::WrongOwner => {
                msg!("Error: Wrong account owner")
            }
            AccessError::AccountNotDeterministic => {
                msg!("Error: Account not generated deterministically")
            }
            AccessError::StakePoolOwnerMustSign => {
                msg!("Error: The stake pool owner must sign")
            }
            AccessError::WrongStakePoolOwner => {
                msg!("Error: Wrong stake pool owner")
            }
            AccessError::StakePoolMustBeEmpty => {
                msg!("Error: The stake pool must be empty")
            }
            AccessError::StakeAccountMustBeEmpty => {
                msg!("Error: The stake account must be empty")
            }
            AccessError::StakeAccountOwnerMustSign => {
                msg!("Error: The stake account owner must sign")
            }
            AccessError::WrongSplTokenProgramId => {
                msg!("Error: Wrong SPL token program ID")
            }
            AccessError::WrongTokenAccountOwner => {
                msg!("Error: Source token account must be owned by SPL Token")
            }
            AccessError::WrongStakeAccountOwner => {
                msg!("Error: Stake account must be program owned")
            }
            AccessError::WrongStakePoolAccountOwner => {
                msg!("Error: Stake pool account must be program owned")
            }
            AccessError::StakeAccountOwnerMismatch => {
                msg!("Error: Stake account mismatch")
            }
            AccessError::StakePoolMismatch => {
                msg!("Error: Stake pool mismatch")
            }
            AccessError::StakePoolVaultMismatch => {
                msg!("Error: Stake pool vault mismatch")
            }
            AccessError::WrongCentralStateAuthority => {
                msg!("Error: Wrong central state authority")
            }
            AccessError::CentralStateAuthorityMustSign => {
                msg!("Error: The central state authority must sign")
            }
            AccessError::Overflow => {
                msg!("Error: Overflow")
            }
            AccessError::NoOp => {
                msg!("Error: operation is a no-op")
            }
            AccessError::WrongMint => {
                msg!("Error: Wrong mint")
            }
            AccessError::WrongCentralVault => {
                msg!("Error: Wrong central vault")
            }
            AccessError::WrongStakePool => {
                msg!("Error: Wrong stake pool")
            }
            AccessError::UnauthorizedSeller => {
                msg!("Error: Unauthorized bond seller")
            }
            AccessError::BondSellerMustSign => {
                msg!("Error: Bond seller must sign")
            }
            AccessError::BondSellerAlreadySigner => {
                msg!("Error: Bond seller has already signed")
            }
            AccessError::NotEnoughSellers => {
                msg!("Error: The bond does not have enough sellers")
            }
            AccessError::BuyerMustSign => {
                msg!("Error: The bond buyer must sign")
            }
            AccessError::WrongQuoteDestination => {
                msg!("Error: Wrong quote token destination")
            }
            AccessError::UnclaimedRewards => {
                msg!("Error: Some rewards need to be claimed first")
            }
            AccessError::CannotUnstake => {
                msg!("Error: Unstake period not over")
            }
            AccessError::InvalidUnstakeAmount => {
                msg!("Error: Invalid unstake amount")
            }
            AccessError::InvalidAmount => {
                msg!("Error: Invalid amount")
            }
            AccessError::InactiveStakePoolNotAllowed => {
                msg!("Error: Inactive stake pool not allowed")
            }
            AccessError::ActiveStakePoolNotAllowed => {
                msg!("Error: Active stake pool not allowed")
            }
            AccessError::InvalidTagChange => {
                msg!("Error: Invalid tag change")
            }
            AccessError::TooManyUnstakeRequests => {
                msg!("Error: Too many unstake requests")
            }
            AccessError::PoolMustBeCranked => {
                msg!("Error: Pool must be cranked")
            }
            AccessError::PendingUnstakeRequests => {
                msg!("Error: Pending unstake request")
            }
            AccessError::CannotStakeZero => {
                msg!("Cannot stake 0 token")
            }
            AccessError::ForbiddenUnlockPeriodZero => {
                msg!("Unlock period cannot be 0")
            }
            AccessError::WrongMplProgram => {
                msg!("Wrong MPL metadata program")
            }
            AccessError::WrongBondAccountOwner => {
                msg!("Wrong bond account owner")
            }
            AccessError::UnsupportedInstruction => {
                msg!("Unsupported instruction")
            }
            AccessError::DeprecatedInstruction => {
                msg!("Deprecated instruction")
            }
            AccessError::TooManyRecipients => {
                msg!("Too many recipients")
            }
            AccessError::NoRecipients => {
                msg!("No recipients")
            }
            AccessError::InvalidPercentages => {
                msg!("Invalid percentages")
            }
            AccessError::InvalidTokenAccount => {
                msg!("Invalid token account")
            }
            AccessError::NonzeroBallance => {
                msg!("Nonzero balance")
            }
            AccessError::DelayTooLong => {
                msg!("Delay too long")
            }
            AccessError::FrozenInstruction => {
                msg!("Frozen instruction")
            }
            AccessError::InvalidRenounceParams => {
                msg!("Invalid renounce params")
            }
        }
    }
}
