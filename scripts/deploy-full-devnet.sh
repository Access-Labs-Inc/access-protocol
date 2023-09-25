mkdir artifacts

echo "STEP 1: Creating an SPL Token"
ts-node create-devnet-spl-token.ts

echo "STEP 2: Deploying the Devnet contract"
./deploy-contract.sh

echo "STEP 3: Initializing the central state"
./init-central-state.sh

echo "STEP 4: Transferring the SPL token authority to the central state"
./spl-set-authority.sh

echo "STEP 5: Migrating the central state to the V2 format"
./migrate-central-state-v2.sh