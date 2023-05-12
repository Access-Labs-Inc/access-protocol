#!/bin/bash
set -x
set -o allexport -o notify

pwd=$(pwd)

PROGRAM_PUBKEY=$(solana-keygen pubkey $pwd/program.json)
AUTHORITY_KEYPAIR="$pwd/authority.json"
TOKEN_PUBKEY=
YEARLY_INFLATION_IN_ACS=2000000000
SOLANA_RPC_PROVIDER_URL=https://api.devnet.solana.com

ts-node init-central-state