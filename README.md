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

`functional.rs` test can be run using Solana program test

```
BPF_OUT_DIR=target/deploy cargo test-bpf --features days-to-sec-10s no-mint-check no-bond-signer --test functional
```

Other Rust tests can be run using

```
BPF_OUT_DIR=target/deploy cargo test-bpf --features no-mint-check no-bond-signer -- --skip functional_10s
```

```

```

### JS

The `js` folder contains the Javascript bindings of the smart contract. This package is published on NPM

```
npm i @accessprotocol/js
```

```
yarn add @accessprotocol/js
```

End to end tests are implemented using `jest`, they can be run using

```
yarn amman:start
yarn jest
```

This will:

- Spawn a local solana test validator via Amman
- Deploy the program
- Run all the instructions of the protocol
- Verify the states of each account at each step

### Devnet deployment

To deploy the program on devnet run the `yarn deploy` command inside the `scripts` folder. This will:

- Create an SPL token with appropriate metadata.
- Build and deploy the Solana program (smart contract).
- Create a `CentralState` data account for the global state of the program.
- Transfer the SPL token authority to the program (central state). 

The following artifacts will be created during the deployment in the `scripts/artifacts` folder:

- `program.json` - Keypair of the program
- `authority.json` - Keypair of the program update authority
- `spl_authority.json` - Keypair of the SPL token authority, not used anymore after the authority transfer
- `central_state_pubkey.txt` - Pubkey of the `CentralState` data account
- `mint_address.txt` - Pubkey of the SPL token mint

### Known shortcomings

- Cannot create two bonds tied to a different pool with the same amount
- Bond functionality is counterintuitive
  - Bond unlocking does not start at `unlock_start_date`, but at `unlock_start_date + unlock_period`
  - If bond is claimed after the `unlock_start_date`, the offset of unlock times is counted relative to this date instead of the `unlock_start_date`
