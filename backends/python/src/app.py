
from flask import Flask, request
from flask_cors import CORS

from errors import ErrorMessage, ErrorType
from flask_jwt_extended import JWTManager, get_jwt_identity, jwt_required
from redis_client import redis_get_nonce, redis_set_nonce
from utils import (ACCESS_TOKEN_SECRET, encode_jwt, generate_nonce,
                   json_response, validate_login, validate_pubkey,
                   verify_nonce, verify_stake)

app = Flask(__name__)

app.config["JWT_SECRET_KEY"] = ACCESS_TOKEN_SECRET
app.config['JWT_IDENTITY_CLAIM'] = 'address'


jwt = JWTManager(app)

CORS(app)


@app.route("/", methods=["GET"])
def home():
    return "visit https://bonfida.org"


@app.route("/auth/nonce", methods=["POST"])
def auth_nonce():
    address = request.get_json()['address']
    if not validate_pubkey(address):
        return json_response(
            False,
            ErrorMessage[ErrorType.InvalidPubkey],
            401
        )
    try:
        nonce = generate_nonce()
        print(f"Address {address} - nonce {nonce}")
        redis_set_nonce(nonce, address)
        return json_response(True, {"nonce": nonce}, 200)
    except:
        return json_response(
            False,
            ErrorMessage[ErrorType.ErrorGeneratingNonce],
            500
        )


@app.route("/auth/login", methods=["POST"])
def auth_login():
    data = request.get_json()
    if not validate_login(data):
        return json_response(
            False,
            ErrorMessage[ErrorType.InvalidRequest],
            401
        )

    address = data["address"]
    signed_nonce = data["signedNonce"]

    nonce = redis_get_nonce(address)

    if nonce is None or not verify_nonce(nonce, signed_nonce, address):
        return json_response(
            False,
            ErrorMessage[ErrorType.InvalidNonce],
            401
        )

    if not (verify_stake()):
        return json_response(
            False,
            ErrorMessage[ErrorType.InvalidStake],
            401
        )

    try:
        return json_response(
            True,
            {"token": encode_jwt(address)},
            200
        )
    except:
        return json_response(
            False,
            ErrorMessage[ErrorType.ErrorValidatingNonce],
            500
        )


@app.route("/article", methods=["GET"])
@jwt_required()
def articles():
    current_address = get_jwt_identity()
    return json_response(
        True,
        {"address": current_address},
        200
    )


if __name__ == '__main__':
    app.run()
