import os
import secrets
from datetime import datetime, timedelta, timezone
from typing import Dict

import jwt
from flask import jsonify

import base58
from dotenv import load_dotenv
from nacl.signing import VerifyKey
from solana import publickey

load_dotenv()

ACCESS_TOKEN_SECRET = os.getenv("ACCESS_TOKEN_SECRET")
REDIS_EXPIRE_TIME = int(os.getenv("REDIS_EXPIRE_TIME"))
JWT_EXPIRE = int(os.getenv("JWT_EXPIRE"))


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


def encode_jwt(address: str) -> bytes:
    now = datetime.now(timezone.utc)
    exp_timestamp = datetime.timestamp(now + timedelta(hours=JWT_EXPIRE))
    return jwt.encode(
        {
            "address": address,
            "exp": int(exp_timestamp)
        },
        ACCESS_TOKEN_SECRET,
    )


def validate_pubkey(address: str) -> bool:
    try:
        publickey.PublicKey(address)
        return True
    except:
        return False


def validate_login(data: Dict[str, str]) -> bool:
    try:
        address = data["address"]
        publickey.PublicKey(address)
        signed_nonce = data["signedNonce"]
        if not isinstance(signed_nonce, str):
            return False
        return True
    except:
        return False


def json_response(success: bool, result: any, status_code: int) -> any:
    return jsonify({"success": success, "result": result}), status_code
