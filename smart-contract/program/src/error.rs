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
