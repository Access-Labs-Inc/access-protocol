# ACCESS Protocol

A Solana smart contract for decentralized content monetization through token staking.

**Program ID:** `6HW8dXjtiTGkD4jzXs7igdFmZExPpmwUrRN5195xGup`

## Overview

Users stake ACCESS tokens into content publisher pools (`StakePool`) via their `StakeAccount` to access content. Token inflation rewards are distributed to stakers and publishers based on the `CentralState` schedule.

**Key components:**
- **CentralState** – Global config: ACCESS mint, inflation schedule, mint authority
- **StakePool** – Publisher pools with circular buffer for balance tracking (updated via permissionless `crank`)
- **StakeAccount** – User deposits into pools; earns yield + grants content access
- **Bonds (v1/v2)** – Locked tokens with linear vesting, can be staked while locked

## Requirements

```
solana-cli 1.18.26
cargo 1.72.0
```

## Project Structure

```
├── smart-contract/
│   ├── program/          # Rust smart contract
│   └── js/               # TypeScript/JS bindings (@accessprotocol/js)
└── scripts/              # Deployment & admin scripts
```

## Building

```bash
make build
```

Output: `smart-contract/program/target/deploy/access_protocol.so`

Or directly:

```bash
cd smart-contract/program
cargo build-bpf
```

### Build Features

| Feature | Description |
|---------|-------------|
| `no-mint-check` | Skip mint address validation |
| `no-bond-signer` | Disable bond signer requirement |
| `v1-instructions-allowed` | Enable legacy v1 instructions |
| `days-to-sec-10s` | 1 day = 10 seconds (testing) |
| `days-to-sec-15m` | 1 day = 15 minutes (testing) |

## Testing

```bash
cd smart-contract/program
make test
```

Or manually:

```bash
# Unit tests
cargo test-bpf --features no-mint-check no-bond-signer v1-instructions-allowed -- --skip functional --skip devnet

# Functional tests (accelerated time)
cargo test-bpf --features no-mint-check no-bond-signer v1-instructions-allowed days-to-sec-10s --test functional
```

## JS Bindings

```bash
npm i @accessprotocol/js
# or
yarn add @accessprotocol/js
```

```typescript
import { stake, unstake, ... } from "@accessprotocol/js"
```

### Building JS bindings

```bash
cd smart-contract/js
yarn install
yarn build
```

## Devnet Deployment

```bash
cd scripts && npm install   # first time only
make deploy-full-devnet RPC=https://api.devnet.solana.com

# Or with v1 instructions enabled:
make deploy-full-devnet-v1 RPC=https://api.devnet.solana.com
```

The `RPC` parameter is required and can be any Solana RPC endpoint.

This will:
1. Create SPL token with metadata
2. Build & deploy the program
3. Initialize `CentralState`
4. Mint initial tokens
5. Transfer mint authority to program
6. Migrate to v2 format

### Deployment Artifacts

Generated in `scripts/artifacts/`:

| File | Description |
|------|-------------|
| `program.json` | Program keypair |
| `authority.json` | Upgrade authority keypair |
| `spl_authority.json` | SPL token authority (pre-transfer) |
| `central_state_pubkey.txt` | CentralState PDA |
| `mint_address.txt` | ACCESS token mint |


## Make Targets

| Target | Description |
|--------|-------------|
| `make build` | Build the program |
| `make deploy-full-devnet RPC=<url>` | Full devnet deployment |
| `make deploy-full-devnet-v1 RPC=<url>` | Deploy with v1 instructions enabled |
| `make check-program` | Verify .so exists |
| `make clean` | Clean build artifacts |

## License

GPL-3.0
