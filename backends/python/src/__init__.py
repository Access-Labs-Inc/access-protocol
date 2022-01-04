from app import app_routes
from flask import Flask

app = Flask(__name__)
ACCESS_TOKEN_SECRET = os.getenv('ACCESS_TOKEN_SECRET')
