<h1 align="center">ACCESS Protocol</h1>

<h2 align="center">Table of content</h2>

1. Concepts
2. Backends
   - Javascript
   - Rust
   - Go
   - Python
3. Smart contract
   - Program
   - Javascript bindings

<h2 align="center">Concepts</h2>

ACCESS Protocol lays the foundation for digital content monetization using a web3 wallet. Users get access to content by staking ACCESS tokens (through their `StakeAccount`) into the content publisher's pool (`StakePool`). ACCESS tokens are distributed to stakers and content publishers through an inflation schedule defined in the `CentralState`.

The protocol also has the possibility to create and sell bonds (`BondAccount`). Bonds allow the protocol to sell locked tokens with linear vesting. The locked tokens can be staked and used to access content of a staked pool.

Publishers need to adapt their backend infrastructures to support authentication and authorization via a web3 wallet. This authentication process relies on signature verifications and a demo example is implemented in the `backends` folder in JS, Rust, Go and Python.

<h2 align="center">Backends</h2>

The `backends` folder contains an example implementation of a REST API using a Solana wallet for authentication and JWT authorization. The example is implemented in Javascript, Rust, Go and Python.

It is strongly recommended to use either the Javascript or Rust implementation as these two language have the best Solana tooling.

<h2 align="center">Smart contract</h2>

The smart contract folder contains two subfolders: `js` and `program`.

### Program

The `program` folder contains the Solana smart contract code, documentation can be generated using Rust doc

```
cargo doc
```

Functional test can be ran using Solana program test

```
cargo test-bpf --features days-to-sec-10s no-mint-check no-bond-signer
```

### JS

The `js` folder contains the Javascript bindings of the smart contract. This package is published on NPM

```
npm i @access-protocol/js
```

```
yarn add @access-protocol/js
```

End to end tests are implemented using `jest`, they can be run using

```
yarn jest
```

This will:

- Spawn a local solana test validator
- Deploy the program
- Run all the instructions of the protocol
- Verify the states of each account at each step
