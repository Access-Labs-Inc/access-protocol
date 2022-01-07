import redis

EXPIRE_TIME: int = 10 * 60  # in seconds

redis_client: redis.Redis = redis.Redis()  # set url


def redis_set_nonce(nonce: str, user_address: str):
    redis_client.set("nonce:" + user_address, nonce, ex=EXPIRE_TIME)


def redis_get_nonce(user_address: str):
    return redis_client.get("nonce:" + user_address)
