use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

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
    #[error("Token account must be owned by SPL Token")]
    WrongTokenAccountOwner,
    #[error("Bond account must be owned by the program")]
    WrongBondAccountOwner,
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
    #[error("Invalid unstake amount")]
    InvalidUnstakeAmount,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Inactive stake pool not allowed")]
    InactiveStakePoolNotAllowed,
    #[error("Active stake pool not allowed")]
    ActiveStakePoolNotAllowed,
    #[error("Invalid tag change")]
    InvalidTagChange,
    #[error("Too many unstake requests")]
    TooManyUnstakeRequests,
    #[error("Pool must be cranked")]
    PoolMustBeCranked,
    #[error("Pending unstake request")]
    PendingUnstakeRequests,
    #[error("Cannot stake 0 token")]
    CannotStakeZero,
    #[error("Unlock period cannot be 0")]
    ForbiddenUnlockPeriodZero,
    #[error("Wrong MPL metadata program")]
    WrongMplProgram,
    #[error("Unsupported instruction")]
    UnsupportedInstruction,
    #[error("Deprecated instruction")]
    DeprecatedInstruction,
    #[error("Too many recipients")]
    TooManyRecipients,
    #[error("No recipients")]
    NoRecipients,
    #[error("Invalid percentages")]
    InvalidPercentages,
    #[error("Invalid token account")]
    InvalidTokenAccount,
    #[error("Nonzero balance")]
    NonzeroBallance,
    #[error("Delay too long")]
    DelayTooLong,
    #[error("Frozen instruction")]
    FrozenInstruction,
    #[error("Invalid renounce params")]
    InvalidRenounceParams,
    #[error("Already renounced")]
    AlreadyRenounced,
    #[error("Royalty account mismatch")]
    RoyaltyAccountMismatch,
    #[error("Royalty ata mismatch")]
    RoyaltyAtaMismatch,
    #[error("Wrong royalty account owner")]
    WrongRoyaltyAccountOwner,
    #[error("Owner must sign")]
    OwnerMustSign,
    #[error("Royalty ata not provided")]
    RoyaltyAtaNotProvided,
    #[error("Royalty ata not deterministic")]
    RoyaltyAtaNotDeterministic,
    #[error("Wrong sysvar instructions id")]
    WrongSysvarInstructionsId,
    #[error("Wrong Access cnft authority")]
    WrongAccessCnftAuthority,
    #[error("Access cnft authority must sign")]
    AccessCnftAuthorityMustSign,
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
