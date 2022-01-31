package routes

import (
	"net/http"

	"github.com/labstack/echo/v4"
)

// Example of a route protected by JWT
func HandleArticle(c echo.Context) error {
	return c.String(http.StatusOK, "Hello, World!")
}
