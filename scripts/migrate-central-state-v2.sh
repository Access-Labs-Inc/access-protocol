#!/bin/bash
set -e
set -o allexport

pwd=$(pwd)
mint_address=$(cat $pwd/artifacts/mint_address.txt)

PROGRAM_PUBKEY=$(solana-keygen pubkey $pwd/artifacts/program.json)
AUTHORITY_KEYPAIR="$pwd/artifacts/authority.json"
MINT_ADDRESS=$mint_address
SOLANA_RPC_PROVIDER_URL="${SOLANA_RPC_PROVIDER_URL:-https://api.devnet.solana.com}"

echo "  Program: $PROGRAM_PUBKEY"
echo "  Mint:    $MINT_ADDRESS"
echo "  Migrating to V2..."

ts-node migrate-central-state-v2 2>&1 | sed 's/^/    /'

echo "  ✓ Migration complete"
