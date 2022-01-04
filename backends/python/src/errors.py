from enum import Enum


class ErrorType(Enum):
    ErrorGeneratingNonce
    InvalidNonce
    InvalidStake
    ErrorValidatingNonce
    InvalidToken


ErrorMessage = {
    ErrorType.ErrorGeneratingNonce: "Error: generating nonce",
    ErrorType.InvalidNonce: "Error: invalid nonce",
    ErrorType.InvalidStake: "Error: invalid stake",
    ErrorType.ErrorValidatingNonce: "Error: validating nonce",
    ErrorType.InvalidToken: "Error: invalid token",
}
