from solana import publickey
from flask_inputs import Inputs
from wtforms.validators import DataRequired, ValidationError


def validate_pubkey(address: str):
    try:
        publickey.Publickey(address)
    except:
        raise ValidationError("Customer does not exist.")


class NonceRequestForm(Inputs):
    address = {"address": [validate_pubkey]}


class LoginRequestForm(Inputs):
    address = {"address": [validate_pubkey]}
    signed_nonce = {"signedNonce": [DataRequired()]}


class AuthorizationHeaderForm(Inputs):
    authorization = {"authorization": [DataRequired()]}
