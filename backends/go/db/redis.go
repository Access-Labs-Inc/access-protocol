package db

import (
	"context"
	"time"

	"github.com/go-redis/redis/v8"
)

var RedisClient *redis.Client

func SetNonce(address string, nonce string) error {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	err := RedisClient.Set(ctx, "nonce:"+address, nonce, 10*time.Minute).Err()
	return err
}

func GetNonce(address string) (string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	return RedisClient.Get(ctx, "nonce:"+address).Result()
}
