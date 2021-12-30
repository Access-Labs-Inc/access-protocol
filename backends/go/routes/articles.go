package routes

import (
	"net/http"

	"github.com/labstack/echo/v4"
)

func HandleArticle(c echo.Context) error {
	return c.String(http.StatusOK, "Hello, World!")
}
