mkdir artifacts

echo "STEP 1: Creating an SPL Token"
ts-node create-devnet-spl-token.ts

echo "STEP 2: Deploying the Devnet contract"
./deploy-contract.sh

echo "STEP 3: Initializing the central state"
./init-central-state.sh

echo "STEP 4: Transferring the SPL token authority to the central state"
./spl-set-authority.sh