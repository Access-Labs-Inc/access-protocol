#!/bin/bash
set -x
set -o allexport -o notify

SOLANA_RPC_PROVIDER_URL=
PROGRAM_ID=
POOL_PUBKEY=
USER_PUBKEY=
PAYER_KEYPAIR=
AMOUNT=10
# Default: 5 minutes from now. Set to 0 for no unlock.
UNLOCK_TIMESTAMP=$(($(date +%s) + 5 * 60))

ts-node drop-bond-v2.ts
