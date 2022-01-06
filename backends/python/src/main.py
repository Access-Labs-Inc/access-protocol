# from validate_req import NonceRequestForm, LoginRequestForm, AuthorizationHeaderForm
# from redis import redis_set_nonce, redis_get_nonce
# from utils import generate_nonce, verify_nonce, encode_jwt
from flask import Flask, request
import os
# from errors import ErrorMessage
exec(open("errors.py").read())
exec(open("redis.py").read())
exec(open("utils.py").read())
exec(open("validate_req.py").read())
# execfile("validate_req.py")

app = Flask(__name__)
ACCESS_TOKEN_SECRET = os.getenv('ACCESS_TOKEN_SECRET')


@app.route('/', methods=['GET'])
def home():
    return 'visit https://bonfida.org'


@app.route('/auth/nonce', methods=['POST'])
def auth_nonce():
    nonce_req_form = NonceRequestForm(request)
    if not(nonce_req_form.validate()):
        return jsonify(success=False, errors=inputs.errors)
    address = request.args.get('address')

    try:
        nonce = generate_nonce()
        print(f"Address {address} - nonce {nonce}")
        redis_set(nonce, address)
        return nonce
    except:
        return ErrorMessage[ErrorType.ErrorGeneratingNonce], 500


@app.route('/auth/login', methods=['POST'])
def auth_login():
    login_req_form = LoginRequestForm(request)
    if not(login_req_form.validate()):
        return jsonify(success=False, errors=inputs.errors)
    address = request.args.get('address')
    signed_nonce = request.args.get('signedNonce')

    nonce = redis_get_nonce(address)
    if nonce == "":
        return "", 400

    print(f"Stored Nonce {nonce}")

    if not(verify_nonce(nonce, signed_nonce, address)):
        return ErrorMessage[ErrorType.InvalidNonce], 401

    if not(verify_stake(address)):
        return ErrorMessage[ErrorType.InvalidStake], 401

    try:
        return encode_jwt(address), 200
    except:
        return ErrorMessage[ErrorType.ErrorValidatingNonce], 500


@app.route('/article', methods=['POST'])
def articles():
    auth_form = AuthorizationHeaderForm(request)
    if not(auth_form.validate()):
        return jsonify(success=False, errors=inputs.errors)
    authorization = request.args.get('authorization')
    token = authorization.split(" ")[1]

    if not(verify_jwt(token)):
        return ErrorMessage[ErrorType.invalid], 403

    return "Successfully logged in", 200
