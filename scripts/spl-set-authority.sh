#!/bin/bash
set -x
set -o allexport -o notify

pwd=$(pwd)
mint_address=$(cat $pwd/artifacts/mint_address.txt)
central_state_pubkey=$(cat $pwd/artifacts/central_state_pubkey.txt)

SPL_AUTHORITY_KEYPAIR="$pwd/artifacts/spl_authority.json"
SOLANA_RPC_PROVIDER_URL=https://api.devnet.solana.com
MINT_ADDRESS=$mint_address
NEW_AUTHORITY_ADDRESS=$central_state_pubkey

ts-node spl-set-authority.ts