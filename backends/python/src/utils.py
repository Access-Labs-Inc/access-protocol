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
import logging
import base64


load_dotenv()

ACCESS_TOKEN_SECRET = os.getenv("ACCESS_TOKEN_SECRET")
REDIS_EXPIRE_TIME = int(os.getenv("REDIS_EXPIRE_TIME"))
JWT_EXPIRE = int(os.getenv("JWT_EXPIRE"))
RPC_URL = os.getenv("RPC_URL")
PROGRAM_ID = "2ZsWiVGXzL4kgMDtSfeEJSV27fBnMptrdcNKKZygUoB8"
STAKE_POOL = "Hs6emyaDnMSxJmGxnHhSmucJh1Q9jSysuKJ5yycWoUuC"
OFFSET_STAKE = 1 + 32
OFFSET_MIN = 1 + 32 + 8 + 32 + 8
POOL_MINIMUM = 100

# Generates a randomly secure 32 bytes nonce
def generate_nonce() -> str:
    return secrets.token_hex(32)

# Verifies a signed nonce
def verify_nonce(nonce: bytes, signed_nonce: str, address: str) -> bool:
    address_bytes = base58.b58decode(address)
    vk = VerifyKey(address_bytes)
    try:
        vk.verify(nonce, bytes.fromhex(signed_nonce))
        return True
    except:
        return False

# Encode a JWT that expires after JWT_EXPIRE
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

# Verifies if a string is a valid base58 encoded public key
def validate_pubkey(address: str) -> bool:
    try:
        publickey.PublicKey(address)
        return True
    except:
        return False

# Validates the login request body
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

# Verifies that a user has enough tokens staked
def check_stake(owner: str) -> bool:
    solana_client = Client(RPC_URL)

    seeds = ["stake_account".encode(), publickey.PublicKey(owner).__bytes__(),  publickey.PublicKey(STAKE_POOL).__bytes__()]
    key = publickey.PublicKey.find_program_address(seeds, publickey.PublicKey(PROGRAM_ID))

    account_info = solana_client.get_account_info(key[0])

    data_bytes = base64.b64decode(account_info["result"]["value"]["data"][0])

    raw_bytes_stake = data_bytes[OFFSET_STAKE: OFFSET_STAKE +8]
    raw_bytes_min = data_bytes[OFFSET_MIN: OFFSET_MIN +8]

    stake = int.from_bytes(raw_bytes_stake, "little")
    min = int.from_bytes(raw_bytes_min, "little")

    return stake > min

# Jsonify a response
def json_response(success: bool, result: any, status_code: int) -> any:
    return jsonify({"success": success, "result": result}), status_code
