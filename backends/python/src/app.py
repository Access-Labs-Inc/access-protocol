from redis_client import redis_set_nonce, redis_get_nonce
from utils import generate_nonce, verify_nonce, encode_jwt, verify_stake, validate_jwt, ACCESS_TOKEN_SECRET, validate_pubkey, validate_login, make_json_response
from flask import Flask, request, jsonify
from errors import ErrorMessage, ErrorType
from flask_cors import CORS
from solana import publickey

app = Flask(__name__)
CORS(app)


@app.route("/", methods=["GET"])
def home():
    return "visit https://bonfida.org"


@app.route("/auth/nonce", methods=["POST"])
def auth_nonce():
    address = request.get_json()['address']
    if not validate_pubkey(address):
        return make_json_response(False, ErrorMessage[ErrorType.InvalidPubkey])
    try:
        nonce = generate_nonce()
        print(f"Address {address} - nonce {nonce}")
        redis_set_nonce(nonce, address)
        return make_json_response(True, {"nonce": nonce}), 200
    except:
        return make_json_response(False, ErrorMessage[ErrorType.ErrorGeneratingNonce]), 500


@app.route("/auth/login", methods=["POST"])
def auth_login():
    data = request.get_json()
    if not validate_login(data):
        return make_json_response(False, ErrorMessage[ErrorType.InvalidRequest]), 401

    address = data["address"]
    signed_nonce = data["signedNonce"]

    nonce = redis_get_nonce(address)

    print(f"Stored Nonce {nonce}")

    if nonce is None or not verify_nonce(nonce, signed_nonce, address):
        return make_json_response(False, ErrorMessage[ErrorType.InvalidNonce]), 401

    if not (verify_stake()):
        return make_json_response(False, ErrorMessage[ErrorType.InvalidStake]), 401

    try:
        return make_json_response(True, {"token": encode_jwt(address)}), 200
    except:
        return make_json_response(False, ErrorMessage[ErrorType.ErrorValidatingNonce]), 500


# @app.route("/article", methods=["POST"])
# def articles():
#     auth_form = AuthorizationHeaderForm(request)
#     if not (auth_form.validate()):
#         return jsonify(success=False, errors=inputs.errors)
#     authorization = request.args.get("authorization")
#     token = authorization.split(" ")[1]

#     if not (validate_jwt(token)):
#         return ErrorMessage[ErrorType.invalid], 403

#     return "Successfully logged in", 200
if __name__ == '__main__':
    app.run()
