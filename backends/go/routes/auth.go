package routes

import (
	"log"

	"github.com/labstack/echo/v4"
)

func HandleNonce(c echo.Context) error {
	log.Println("COucou")
	return nil
}
