use num_derive::FromPrimitive;
use thiserror::Error;

use solana_program::{decode_error::DecodeError, program_error::ProgramError};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum AccessError {
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
    #[error("Source token account must be owned by SPL Token")]
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
    #[error("Wrong central state authority")]
    WrongCentralStateAuthority,
    #[error("The central state authority must sign")]
    CentralStateAuthorityMustSign,
    #[error("Overflow")]
    Overflow,
    #[error("Operation is a no-op")]
    NoOp,
    #[error("Wrong mint")]
    WrongMint,
    #[error("Wrong central vault")]
    WrongCentralVault,
    #[error("Wrong stake pool")]
    WrongStakePool,
    #[error("Unauthorized seller")]
    UnauthorizedSeller,
    #[error("Bond seller must sign")]
    BondSellerMustSign,
    #[error("Bond seller has already signed")]
    BondSellerAlreadySigner,
    #[error("The bond does not have enough sellers")]
    NotEnoughSellers,
    #[error("The bond buyer must sign")]
    BuyerMustSign,
    #[error("Wrong quote token destination")]
    WrongQuoteDestination,
    #[error("Rewards must be claimed first")]
    UnclaimedRewards,
    #[error("Unstake period not over")]
    CannotUnstake,
    #[error("Inactive stake pool not allowed")]
    InactiveStakePoolNotAllowed,
    #[error("Invalid tag change")]
    InvalidTagChange,
    #[error("Too many unstake requests")]
    TooManyUnstakeRequests,
    #[error("Pool must be cranked")]
    PoolMustBeCranked,
    #[error("Pending unstake request")]
    PendingUnstakeRequests,
}

impl From<AccessError> for ProgramError {
    fn from(e: AccessError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for AccessError {
    fn type_of() -> &'static str {
        "AccessError"
    }
}
