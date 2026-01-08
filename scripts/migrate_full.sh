#!/bin/bash
set -x
set -o allexport -o notify
set -e

PROGRAM_ID=9LPrKE24UaN9Bsf5rXCS4ZGor9VmjAUxkLCMKHr73sdV
PROGRAM_PUBKEY=9LPrKE24UaN9Bsf5rXCS4ZGor9VmjAUxkLCMKHr73sdV
MINT_ADDRESS=5hGLVuE4wHW8mcHUJKEyoJYeg653bj8nZeXgUJrfMxFC
AUTHORITY_KEYPAIR=/Users/matusvla/go/src/github.com/Access-Labs-Inc/access-protocol/scripts/temp.json
NETWORK=devnet
SOLANA_RPC_PROVIDER_URL=https://devnet.helius-rpc.com/?api-key=6b3f6b84-a89d-485b-bea8-5cb861db8161

# STEP 2 Deploy new program
./deploy-contract.sh
# TODO allow ledger

# STEP 3 Migrate Central state
ts-node migrate-central-state-v2.ts

# STEP 4 Setup fee split

# STEP 5 OPTIONAL - set freeze authority