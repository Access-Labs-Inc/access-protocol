package types

import (
	"github.com/golang-jwt/jwt"
)

type ApiResponse struct {
	Success bool        `json:"success"`
	Result  interface{} `json:"result"`
}

func NewApiResonse(success bool, result interface{}) ApiResponse {
	return ApiResponse{Success: success, Result: result}
}

type NonceRequestBody struct {
	Address string `json:"address"`
}

type NonceResult struct {
	Nonce string `json:"nonce"`
}

type LoginRequestBody struct {
	Address     string `json:"address"`
	SignedNonce string `json:"signedNonce"`
}

type LoginResut struct {
	Token string `json:"token"`
}

type JWT struct {
	Address string `json:"address"`
	Iat     int64  `json:"iat"`
	jwt.StandardClaims
}
