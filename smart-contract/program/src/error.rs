use num_derive::FromPrimitive;
use thiserror::Error;

use solana_program::{decode_error::DecodeError, program_error::ProgramError};

pub type MediaResult<T = ()> = Result<T, MediaError>;

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum MediaError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
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
