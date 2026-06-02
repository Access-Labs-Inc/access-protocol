#!/bin/bash
set -x
set -o allexport -o notify

SOLANA_RPC_PROVIDER_URL=
PROGRAM_ID=

# Owner of the stake pool (must sign create_stake_pool)
OWNER_KEYPAIR=
# Optional. If empty, OWNER_KEYPAIR is used as payer.
PAYER_KEYPAIR=

# Raw token amount in base units (ACS has 6 decimals; e.g. 10 ACS = 10000000)
MINIMUM_STAKE_AMOUNT=10000000

ts-node create-stake-pool.ts