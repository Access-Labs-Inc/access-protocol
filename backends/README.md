<h1 align="center">Access protocol server</h1>

<h2 align="center">Table of content</h2>

1. Overview
2. Rust server
3. Javascript server
4. Python server
5. Go server

⚠️ This code is made for development purposes only. Do not use in a production deployment

<h2 align="center">Overview</h2>

This repository shows how an authentication/authorization system can be implemented using the Access protocol. A user can identify himself with his Solana wallet by signing a random message (nonce) that was generated on the server. If the the signature is verified the user will receive a JSON Web Token (JWT) that will allow him to access protected content.

This example is implemented in Rust, Javacript, Python and Go and includes the three following endpoints:

- `/auth/nonce`:

  - Generates a randomly secure 32 bytes nonce
  - Stores the nonce with a TTL of `x` seconds in Redis
  - Returns the nonce
  - Payload format:

  ```json
  {
    "address": "agsWhfJ5PPGjmzMieWY8BR5o1XRVszUBQ5uFz4CtDiJ"
  }
  ```

  - Response format:

  ```json
  {
    "success": true,
    "result": {
      "nonce": "82da3d22c51d04432f1e9eea7c805a5262269fbb19bcceeec24ace20bbfa1254"
    }
  }
  ```

- `/auth/login`:

  - Verifies a signed nonce
  - Generates a JWT
  - Payload format:

  ```js
  {
    "address": "agsWhfJ5PPGjmzMieWY8BR5o1XRVszUBQ5uFz4CtDiJ",
    "signedNonce": "1132dc8542f5f40b6053f1bee3adb4d355e9edc226fdc60e519e17a5eff77bfba16c86f2c012368852bb14a67a314ca0eaf13495a4cf2c2020996f8f03375707"
  }
  ```

  - Response format:

  ```js
  {
  success: true,
  result: {
    token:
      "eyJhbGciOiJIUzI1NiJ9.eyJhZGRyZXNzIjoiRzM0S2FqU1ZiS1NNbzF2ZDdkTWdYQzk5M0FxYTRqMVN5dGloS0tzRFhESE4iLCJpYXQiOiIxNjQyNjY3ODE3In0.7s5vsBpRelznjMx-dQww51w9iUFDLsJFJ0K9uSe8beQ",
  },
  }
  ```

  - `/article`:
    - JWT protected endpoint

Errors will have the following format and a status code in the header:

```js
{
    "success": false,
    "result": "Error message"
}
```

<p align="center">
<img width="100%" src="./assets/auth.png"/>
</p>

<h2 align="center">Rust server</h2>

The Rust example is implemented with the [Actix web](https://actix.rs/) framework.

Run the code on your local machine with

```
cargo run
```

Deployment can be done with systemd or Docker.

The following environment variables must be defined in the `.env` file:

```
ACCESS_TOKEN_SECRET= JWT secret token
REDIS_URL= The Redis URL
RPC_URL= The Solana RPC URL used to fetch blockchain data e.g https://api.devnet.solana.com
STAKE_POOL_KEY=The public key of your stake pool
PROGRAM_ID= The ACCESS program ID
```

Other setting variables must be defined in `settings.rs`:

```rust
/*
    Redis settings
*/
pub const REDIS_EXPIRE: usize = 60 * 10; // Redis TTL

/*
    JWT settings
*/
pub const JWT_EXPIRE: u64 = 24 * 60 * 60; // TTL of JWTs
```

<h2 align="center">Javascript server</h2>

The Javascript example is implemented with the [Express](https://expressjs.com/) framework.

Run the code on your local machine with

```
yarn dev
```

Deployment can be done with pm2 or Docker.

The following environment variables must be defined in the `.env` file:

```
ACCESS_TOKEN_SECRET= JWT secret token
EXPIRATION_INTERVAL= The TTL of JWTs
REDIS_EXPIRE_TIME= The redis TTL for nonces
RPC_URL= The Solana RPC URL used to fetch blockchain data e.g https://api.devnet.solana.com
STAKE_POOL_KEY= The public key of your stake pool
```

<h2 align="center">Python server</h2>

The Python example is implemented with the [Flask](https://flask.palletsprojects.com/en/2.0.x/) framework.

Run the code on your local machine with Docker

```
docker-compose up
```

Deployment can be done with Docker.

The following environment variables must be defined in the the `docker-compose.yml` file:

```
    environment:
      - ACCESS_TOKEN_SECRET= JWT secret token
      - REDIS_EXPIRE_TIME= The redis TTL for nonces
      - JWT_EXPIRE= The TTL of JWTs
      - RPC_URL= he Solana RPC URL used to fetch blockchain data e.g https://api.devnet.solana.com
```

<h2 align="center">Go server</h2>

The Go example is implemented with the [Echo](https://echo.labstack.com/) framework.

```
docker-compose up
```

Deployment can be done with Docker.

The following environment variables must be defined in the `.env` file:

```
ACCESS_TOKEN_SECRET=
REDIS_URL= Redis URL (can be set to 'redis' and docker will do the mapping)
STAKE_POOL_KEY= The public key of your stake pool
PROGRAM_ID= The ACCESS program ID
```

Redis TTL can be defined in `redis.go`:

```go
// Redis client
const TTL = 10 // TTL in minutes
```
