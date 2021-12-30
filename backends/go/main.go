package main

import (
	"access_backend/routes"
	"access_backend/utils"

	"context"
	"log"

	"github.com/go-redis/redis/v8"
	"github.com/joho/godotenv"
	"github.com/labstack/echo/v4"
)

var ctx = context.Background()

func main() {
	err := godotenv.Load()

	rdb := redis.NewClient(&redis.Options{
		Addr:     "localhost:6379",
		Password: "", // no password set
		DB:       0,  // use default DB
	})

	err = rdb.Set(ctx, "key", "value", 0).Err()
	if err != nil {
		panic(err)
	}

	if err != nil {
		log.Fatal("Error loading .env file")
	}

	e := echo.New()
	e.GET("/", routes.HandleNonce)
	e.GET("/articles", routes.HandleArticle, utils.ValidateToken)
	e.Logger.Fatal(e.Start(":3001"))
}
