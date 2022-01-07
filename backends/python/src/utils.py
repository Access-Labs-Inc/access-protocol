from datetime import timezone, datetime
import jwt
import base58
from nacl.encoding import Base64Encoder
from nacl.signing import VerifyKey
import secrets

from main import ACCESS_TOKEN_SECRET


def generate_nonce() -> str:
    return secrets.token_hex(16)


def verify_nonce(nonce: str, signed_nonce: str, address: str) -> bool:
    address_bytes = base58.b58decode(address)
    vk = VerifyKey(
        address_bytes,
    )
    try:
        VerifyKey.verify(nonce, signed_nonce.message, encoder=Base64Encoder)
        return True
    except:
        return False


def verify_stake() -> bool:
    return True  # TODO


# JWT
JWT_EXPIRATION_INTERVAL: int = 24 * 60 * 60


def encode_jwt(address: str) -> bytes:
    return jwt.encode(
        {
            "some": address,
            "exp": datetime.datetime.now(tz=timezone.utc)
            + datetime.timedelta(seconds=JWT_EXPIRATION_INTERVAL),
        },
        ACCESS_TOKEN_SECRET,
    )


def validate_jwt(token: str) -> bool:
    expiry_date = jwt.decode(token, ACCESS_TOKEN_SECRET)["exp"]
    return expiry_date > datetime.datetime.now(tz=timezone.utc)
