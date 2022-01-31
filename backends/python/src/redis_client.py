import redis

from utils import REDIS_EXPIRE_TIME

NONCE_PREFIX = "nonce:"

redis_client: redis.Redis = redis.Redis(
    host='redis', port=6379, db=0)  # set url

# Stores a nonce in redis with a TTL of REDIS_EXPIRE_TIME
def redis_set_nonce(nonce: str, user_address: str):
    redis_client.set(NONCE_PREFIX + user_address, nonce, ex=REDIS_EXPIRE_TIME)

# Returns the latest nonce generated for a user
def redis_get_nonce(user_address: str):
    return redis_client.get(NONCE_PREFIX + user_address)
