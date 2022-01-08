import redis

EXPIRE_TIME: int = 10 * 60  # in seconds

NONCE_PREFIX = "nonce:"

redis_client: redis.Redis = redis.Redis(
    host='redis', port=6379, db=0)  # set url


def redis_set_nonce(nonce: str, user_address: str):
    redis_client.set(NONCE_PREFIX + user_address, nonce, ex=EXPIRE_TIME)


def redis_get_nonce(user_address: str):
    return redis_client.get(NONCE_PREFIX + user_address)
