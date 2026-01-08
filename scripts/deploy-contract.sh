#!/bin/bash
set -e

pwd=$(pwd)
NETWORK=${NETWORK:-devnet}
ALLOW_V1=${ALLOW_V1:-false}
# Use RPC URL if set, otherwise use network name (solana CLI accepts both)
RPC_URL="${SOLANA_RPC_PROVIDER_URL:-$NETWORK}"

pushd ../smart-contract/program > /dev/null

echo "  Building smart contract..."
# IMPORTANT: If not no-mint-check present you need to update MINT_ADDRESS in (state.rs)
FEATURES=""
if [ "$ALLOW_V1" == "true" ]; then
  FEATURES="no-bond-signer no-mint-check v1-instructions-allowed"
  echo "    Features: $FEATURES"
fi
cargo build-bpf --features ${FEATURES} 2>&1 | grep -E "(Compiling|Finished|warning:|error)" || true

PROGRAM_KEYPAIR=${PROGRAM_KEYPAIR:-"$pwd/artifacts/program.json"}
if test -f "$PROGRAM_KEYPAIR"; then
  echo "  Program keypair: exists"
else
  echo "  Creating program keypair..."
  solana-keygen new --outfile $PROGRAM_KEYPAIR --no-bip39-passphrase --silent
fi

AUTHORITY_KEYPAIR=${AUTHORITY_KEYPAIR:-"$pwd/artifacts/authority.json"}
if test -f "$AUTHORITY_KEYPAIR"; then
  echo "  Authority keypair: exists"
else
  if [ "$NETWORK" == "devnet" ]; then
    echo "  Creating authority keypair..."
    solana-keygen new --outfile $AUTHORITY_KEYPAIR --no-bip39-passphrase --silent
  else
    echo "  ✗ Error: AUTHORITY_KEYPAIR required for $NETWORK"
    exit 1
  fi
fi

solana config set --keypair $AUTHORITY_KEYPAIR > /dev/null

REQUIRED_SOL=6
authority_address=$(solana address)

check_balance() {
  solana balance -u ${RPC_URL} | grep -Eo '^[0-9]+\.?[0-9]*'
}

balance=$(check_balance)
echo "  Balance: ${balance} SOL (need ${REQUIRED_SOL})"

if (( $(echo "$balance >= $REQUIRED_SOL" | bc -l) )); then
  echo "  ✓ Balance sufficient"
else
  if [ "$NETWORK" == "devnet" ]; then
    echo "  Requesting airdrop..."
    if solana airdrop 2 -u ${RPC_URL} > /dev/null 2>&1; then
      sleep 2
      balance=$(check_balance)
      echo "  Balance after airdrop: ${balance} SOL"
    else
      echo "  ⚠ Airdrop failed (rate limited)"
    fi
    
    balance=$(check_balance)
    if (( $(echo "$balance < $REQUIRED_SOL" | bc -l) )); then
      echo ""
      echo "  ════════════════════════════════════════════════════════════"
      echo "    Insufficient balance: ${balance} SOL (need ${REQUIRED_SOL} SOL)"
      echo "    Please fund this wallet manually:"
      echo ""
      echo "      ${authority_address}"
      echo ""
      echo "    Use: https://faucet.solana.com"
      echo "    Then press ENTER to continue..."
      echo "  ════════════════════════════════════════════════════════════"
      read -r
      
      balance=$(check_balance)
      if (( $(echo "$balance < $REQUIRED_SOL" | bc -l) )); then
        echo "  ✗ Still insufficient balance: ${balance} SOL"
        exit 1
      fi
    fi
  else
    echo "  ✗ Insufficient balance for ${NETWORK}. Need ${REQUIRED_SOL} SOL, have ${balance} SOL"
    echo "    Fund this wallet: ${authority_address}"
    exit 1
  fi
  echo "  ✓ Balance sufficient: ${balance} SOL"
fi

program_pubkey=$(solana-keygen pubkey ${PROGRAM_KEYPAIR})
authority_pubkey=$(solana-keygen pubkey ${AUTHORITY_KEYPAIR})
echo "  Program:   $program_pubkey"
echo "  Authority: $authority_pubkey"
echo "  Deploying to ${NETWORK}..."

solana program deploy ../../so/access_protocol.so \
  --program-id ${PROGRAM_KEYPAIR} \
  --upgrade-authority ${AUTHORITY_KEYPAIR} \
  --keypair ${AUTHORITY_KEYPAIR} \
  -u ${RPC_URL} 2>&1 | grep -v "^$" | sed 's/^/    /'

popd > /dev/null
echo "  ✓ Contract deployed"
