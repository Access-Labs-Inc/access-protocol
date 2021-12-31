package utils

import (
	"fmt"
	"net/http"
	"strings"

	"os"

	"github.com/golang-jwt/jwt"
	"github.com/labstack/echo/v4"
)

func keyFunc(t *jwt.Token) (interface{}, error) {
	signingKey := []byte(os.Getenv("ACCESS_TOKEN"))
	if t.Method.Alg() != "HS256" {
		return nil, fmt.Errorf("unexpected jwt signing method=%v", t.Header["alg"])
	}
	return signingKey, nil
}

func ValidateToken(next echo.HandlerFunc) echo.HandlerFunc {
	return func(c echo.Context) error {
		authToken := c.Request().Header.Get("authorization")
		splitted := strings.Split(authToken, " ")
		if len(splitted) != 2 {
			c.Logger().Error("Invalid authorization header")
			return c.String(http.StatusForbidden, "Invalid autorization header")
		}
		bearerToken := splitted[1]

		_, err := jwt.Parse(bearerToken, keyFunc)

		if err != nil {
			return c.String(http.StatusForbidden, "Invalid autorization header")
		}

		return next(c)
	}

}
