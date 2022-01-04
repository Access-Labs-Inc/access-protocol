import secrets
from nacl.signing import VerifyKey
import base58
import jwt


def generate_nonce():
    return secrets.token_hex(16)


def verify_nonce(nonce, signed_nonce, address):
    address_bytes = base58.b58decode(address)
    vk = VerifyingKey(address_bytes, )
    try:
        verify_key.verify(nonce, signed_nonce.message,
                          encoder=Base64Encoder)
        return True
    except:
        return False


def verify_stake():
    return True     # TODO


# JWT
JWT_EXPIRATION_INTERVAL = 24 * 60 * 60


def encode_jwt(address):
    return jwt.encode({"some": address, "exp": datetime.datetime.now(
        tz=timezone.utc) + datetime.timedelta(seconds=JWT_EXPIRATION_INTERVAL)}, ACCESS_TOKEN_SECRET)


def validate_jwt(token):
    expiry_date = jwt.decode(token,  ACCESS_TOKEN_SECRET)["exp"]
    return expiry > datetime.datetime.now(
        tz=timezone.utc)
