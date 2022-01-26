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
from solana.rpc.api import Client
import numpy

load_dotenv()

ACCESS_TOKEN_SECRET = os.getenv("ACCESS_TOKEN_SECRET")
REDIS_EXPIRE_TIME = int(os.getenv("REDIS_EXPIRE_TIME"))
JWT_EXPIRE = int(os.getenv("JWT_EXPIRE"))
RPC_URL = os.getenv("RPC_URL")
PROGRAM_ID = ""
STAKE_POOL = ""
OFFSET = 1 + 32
POOL_MINIMUM = 100


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

def check_stake(owner: str) -> bool:
    solana_client = Client(RPC_URL)

    seeds = ["stake_account".encode(), owner.encode(),  STAKE_POOL.encode()]
    key = publickey.PublicKey.find_program_address(seeds, publickey.PublicKey(PROGRAM_ID))
    
    account_info = solana_client.get_account_info(key)

    raw_bytes = account_info["result"]["value"]["data"][OFFSET: OFFSET +8]

    stake = numpy.uint64(raw_bytes)

    return stake > POOL_MINIMUM