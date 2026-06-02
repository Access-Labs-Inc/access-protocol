#!/bin/bash
set -x
set -o allexport -o notify

SOLANA_RPC_PROVIDER_URL=
PROGRAM_ID=

# Option A: direct BondV2 account PDA (leave empty to use Option B)
BOND_ACCOUNT_PUBKEY=

# Option B: derive BondV2 account PDA from owner + pool + unlock timestamp
USER_PUBKEY=
POOL_PUBKEY=
# Use the same unlock timestamp used when creating the bond. 0 means no unlock timestamp.
UNLOCK_TIMESTAMP=

# Fee payer keypair path
PAYER_KEYPAIR=

ts-node unlock-bond-v2.ts