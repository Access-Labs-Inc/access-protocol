from datetime import timezone, datetime, timedelta
import jwt
import time
import base58
from nacl.encoding import Base64Encoder, HexEncoder, URLSafeBase64Encoder
from nacl.signing import VerifyKey
import secrets
import os
from solana import publickey
from dotenv import load_dotenv
from flask import jsonify

load_dotenv()

ACCESS_TOKEN_SECRET = os.getenv("ACCESS_TOKEN_SECRET")


def generate_nonce() -> str:
    return secrets.token_hex(16)


def verify_nonce(nonce: bytes, signed_nonce: str, address: str) -> bool:
    address_bytes = base58.b58decode(address)
    vk = VerifyKey(address_bytes)
    try:
        vk.verify(nonce, bytes.fromhex(signed_nonce))
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
            "iat": time.time()
        },
        ACCESS_TOKEN_SECRET,
    )


def validate_jwt(token: str) -> bool:
    expiry_date = jwt.decode(token, ACCESS_TOKEN_SECRET)["exp"]
    return expiry_date > datetime.now(tz=timezone.utc)


def validate_pubkey(address: str) -> bool:
    try:
        publickey.PublicKey(address)
        return True
    except:
        return False


def validate_login(data: dict[str, str]) -> bool:
    try:
        address = data["address"]
        publickey.PublicKey(address)
        signed_nonce = data["signedNonce"]
        if not isinstance(signed_nonce, str):
            return False
        return True
    except:
        return False


def make_json_response(success: bool, result: any) -> any:
    return jsonify({"success": success, "result": result})
