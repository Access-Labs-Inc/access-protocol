package utils

import (
	"crypto/ed25519"
	"encoding/hex"

	"github.com/mr-tron/base58"

	"github.com/mazen160/go-random"
)

func GenerateNonce() (string, error) {
	var nonce string
	bytes, err := random.Bytes(32)
	if err != nil {
		return "", err
	}
	nonce = hex.EncodeToString(bytes)
	return nonce, nil
}

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
