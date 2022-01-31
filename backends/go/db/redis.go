package db

import (
	"context"
	"time"

	"github.com/go-redis/redis/v8"
)

// Redis client
var RedisClient *redis.Client

// Set a nonce for a user with a TTL of 10 minutes
func SetNonce(address string, nonce string) error {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	err := RedisClient.Set(ctx, "nonce:"+address, nonce, 10*time.Minute).Err()
	return err
}

// Get the latest nonce generated for a user
func GetNonce(address string) (string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	return RedisClient.Get(ctx, "nonce:"+address).Result()
}
