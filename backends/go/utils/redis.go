package utils

import (
	"github.com/go-redis/redis/v8"
)

type RedisKey string

const (
	Nonce RedisKey = "nonce:"
	Stake          = "stake:"
)

func SetNonce(client *redis.Client, key string) error {
	return nil
}
