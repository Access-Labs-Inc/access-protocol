#!/bin/bash
set -x
set -o allexport -o notify

pwd=$(pwd)

SPL_AUTHORITY_KEYPAIR="$pwd/spl-authority.json"
SOLANA_RPC_PROVIDER_URL=https://api.devnet.solana.com
MINT_ADDRESS=
NEW_AUTHORITY_ADDRESS=


ts-node spl-set-authority.ts