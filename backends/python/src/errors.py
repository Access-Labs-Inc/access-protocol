from enum import Enum


class ErrorType(Enum):
    ErrorGeneratingNonce = 0
    InvalidNonce = 1
    InvalidStake = 2
    ErrorValidatingNonce = 3
    InvalidToken = 4
    InvalidPubkey = 5
    InvalidRequest = 6


ErrorMessage = {
    ErrorType.ErrorGeneratingNonce: "Error: generating nonce",
    ErrorType.InvalidNonce: "Error: invalid nonce",
    ErrorType.InvalidStake: "Error: invalid stake",
    ErrorType.ErrorValidatingNonce: "Error: validating nonce",
    ErrorType.InvalidToken: "Error: invalid token",
    ErrorType.InvalidPubkey: "Error: invalid public key",
    ErrorType.InvalidRequest: "Error: invalid request parameters",
}
