package utils

import (
	"crypto/ed25519"
	"encoding/hex"

	"github.com/mr-tron/base58"

	"github.com/mazen160/go-random"
)

// Generates a randomly secure 32 bytes nonce
func GenerateNonce() (string, error) {
	var nonce string
	bytes, err := random.Bytes(32)
	if err != nil {
		return "", err
	}
	nonce = hex.EncodeToString(bytes)
	return nonce, nil
}

// Verifies if a string is a valid 32 bytes nonce
func IsValidNonce(nonce string) bool {
	bytes, err := hex.DecodeString(nonce)
	if err != nil {
		return false
	}
	if len(bytes) != 32 {
		return false
	}
	return true
}
// Verifies a signed nonce
// - nonce: hex encoded nonce
// - signedNonce: signed nonce (i.e signature to verify)
// - pubkeyString: Public key used to sign the nonce
func VerifyNonce(nonce string, signedNonce string, pubkeyString string) bool {
	bytes, err := base58.Decode(pubkeyString)
	if err != nil {
		return false
	}

	nonceAsBytes := []byte(nonce)
	if err != nil {
		return false
	}

	signedNonceAsBytes, err := hex.DecodeString(signedNonce)
	if err != nil {
		return false
	}

	return ed25519.Verify(bytes, nonceAsBytes, signedNonceAsBytes)
}
