package main

import (
	"access_backend/db"
	"access_backend/routes"
	"access_backend/utils"

	"context"
	"log"

	"github.com/go-redis/redis/v8"
	"github.com/joho/godotenv"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
)

var ctx = context.Background()

func main() {
	err := godotenv.Load()
	db.RedisClient = redis.NewClient(&redis.Options{
		Addr:     "redis:6379",
		Password: "", // no password set
		DB:       0,  // use default DB
	})

	if err != nil {
		log.Fatal("Error loading .env file")
	}

	e := echo.New()

	// CORS
	e.Use(middleware.CORSWithConfig(middleware.CORSConfig{
		AllowOrigins: []string{"http://localhost:3000"},
		AllowHeaders: []string{echo.HeaderOrigin, echo.HeaderContentType, echo.HeaderAccept},
	}))

	/// Auth endpoints
	e.POST("/auth/nonce", routes.HandleNonce)
	e.POST("/auth/login", routes.HandleLogin)

	/// Endpoint protected by JWT
	e.GET("/article", routes.HandleArticle, utils.ValidateToken)

	e.Logger.Fatal(e.Start(":3001"))
}
