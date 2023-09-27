#!/bin/bash
set -x
set -o allexport -o notify
set -e

#mkdir -p artifacts
#
#echo "STEP 1: Creating an SPL Token"
#ts-node create-devnet-spl-token.ts
#
#echo "STEP 2: Deploying the Devnet contract with V1 instructions allowed"
## the ALLOW_V1 flag is only needed if we want to keep the V1 instructions enabled
## ALLOW_V1=true ./deploy-contract.sh
#./deploy-contract.sh
#
#echo "STEP 3: Initializing the central state"
#./init-central-state.sh

echo "STEP 4: Minting 1,000,000 tokens to token bank as minting is disabled in V2 (if ALLOW_V1=false)"
./spl-mint.sh

echo "STEP 5: Transferring the SPL token authority to the central state"
./spl-set-authority.sh

echo "STEP 6: Migrating the central state to the V2 format"
./migrate-central-state-v2.sh