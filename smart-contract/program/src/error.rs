use num_derive::FromPrimitive;
use thiserror::Error;

use solana_program::{decode_error::DecodeError, program_error::ProgramError};

pub type MediaResult<T = ()> = Result<T, MediaError>;

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum MediaError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong system program key")]
    WrongSystemProgram,
    #[error("Wrong rent sysvar key")]
    WrongRent,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account not generated deterministically")]
    AccountNotDeterministic,
    #[error("The stake pool owner must sign")]
    StakePoolOwnerMustSign,
    #[error("Wrong stake pool owner")]
    WrongStakePoolOwner,
    #[error("The stake pool must be empty")]
    StakePoolMustBeEmpty,
    #[error("The stake account must be empty")]
    StakeAccountMustBeEmpty,
    #[error("The stake account owner must sign")]
    StakeAccountOwnerMustSign,
    #[error("Wrong SPL token program ID")]
    WrongSplTokenProgramId,
    #[error("Source token account must be owned to SPL Token")]
    WrongTokenAccountOwner,
    #[error("Stake account must be program owned")]
    WrongStakeAccountOwner,
    #[error("Stake pool account must be program owned")]
    WrongStakePoolAccountOwner,
    #[error("Stake account owner mismatch")]
    StakeAccountOwnerMismatch,
    #[error("Stake pool mismatch")]
    StakePoolMismatch,
    #[error("Stake pool vault mismatch")]
    StakePoolVaultMismatch,
}

impl From<MediaError> for ProgramError {
    fn from(e: MediaError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MediaError {
    fn type_of() -> &'static str {
        "MediaError"
    }
}
