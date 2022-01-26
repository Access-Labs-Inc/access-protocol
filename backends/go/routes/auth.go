package routes

import (
	"access_backend/db"
	"access_backend/types"
	"access_backend/utils"
	"net/http"

	"encoding/json"

	"time"

	"os"

	"github.com/golang-jwt/jwt"
	"github.com/labstack/echo/v4"
)

func HandleNonce(c echo.Context) error {
	var parsedBody types.NonceRequestBody
	err := json.NewDecoder(c.Request().Body).Decode(&parsedBody)

	if err != nil {
		return c.JSON(http.StatusBadRequest, types.NewApiResonse(false, "Invalid body"))
	}

	nonce, err := utils.GenerateNonce()

	db.SetNonce(parsedBody.Address, nonce)

	if err != nil {
		return c.JSON(http.StatusInternalServerError, types.NewApiResonse(false, err))
	}

	return c.JSON(http.StatusOK, types.NewApiResonse(true, types.NonceResult{Nonce: nonce}))
}

func HandleLogin(c echo.Context) error {
	var parsedBody types.LoginRequestBody
	err := json.NewDecoder(c.Request().Body).Decode(&parsedBody)

	if err != nil {
		return c.JSON(http.StatusBadRequest, types.NewApiResonse(false, "Invalid body"))
	}

	nonce, err := db.GetNonce(parsedBody.Address)

	if err != nil {
		return c.JSON(http.StatusBadRequest, types.NewApiResonse(false, "Invalid nonce"))
	}

	if !utils.VerifyNonce(nonce, parsedBody.SignedNonce, parsedBody.Address) {
		return c.JSON(http.StatusBadRequest, types.NewApiResonse(false, "Invalid signature"))
	}

	// Check stake amount
	if !utils.CheckStake(parsedBody.Address){
		return c.JSON(http.StatusBadRequest, types.NewApiResonse(false, "Not enough stake"))
	}

	// Sign JWT
	claims := types.JWT{Address: parsedBody.Address, Iat: time.Now().Unix()}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	userToken, err := token.SignedString([]byte(os.Getenv("ACCESS_TOKEN")))

	if err != nil {
		return c.JSON(http.StatusInternalServerError, types.NewApiResonse(false, "Error signing token"))
	}

	return c.JSON(http.StatusOK, types.NewApiResonse(true, types.LoginResut{Token: userToken}))
}
