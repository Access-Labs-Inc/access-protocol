#!/bin/bash
set -x
set -o allexport -o notify

pwd=$(pwd)
mint_address=$(cat $pwd/artifacts/mint_address.txt)
echo "mint_address: $mint_address"

PROGRAM_PUBKEY=$(solana-keygen pubkey $pwd/artifacts/program.json)
AUTHORITY_KEYPAIR="$pwd/artifacts/authority.json"
MINT_ADDRESS=$mint_address
YEARLY_INFLATION_IN_ACS=2000000000
SOLANA_RPC_PROVIDER_URL=https://api.devnet.solana.com

ts-node init-central-state