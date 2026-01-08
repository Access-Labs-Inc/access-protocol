#!/bin/bash
set -e
set -o allexport

pwd=$(pwd)
SOLANA_RPC_PROVIDER_URL="${SOLANA_RPC_PROVIDER_URL:-https://api.devnet.solana.com}"
MINT_ADDRESS=$(cat $pwd/artifacts/mint_address.txt)

# Create token bank keypair if needed
if test -f "$pwd/artifacts/token_bank.json"; then
  echo "  Token bank keypair: exists"
else
  echo "  Creating token bank keypair..."
  solana-keygen new --outfile $pwd/artifacts/token_bank.json --no-bip39-passphrase --silent
fi

PUBKEY=$(solana-keygen pubkey $pwd/artifacts/token_bank.json)
echo "  Token bank: $PUBKEY"
echo "  Mint:       $MINT_ADDRESS"

retry_cmd() {
  local timeout=30
  local start=$(date +%s)
  local output
  while true; do
    if output=$("$@" 2>&1); then
      echo "$output" | grep -v "^$" | head -3 | sed 's/^/    /'
      return 0
    fi
    local now=$(date +%s)
    if (( now - start >= timeout )); then
      echo "  ✗ Command failed after ${timeout}s"
      echo "$output" | sed 's/^/    /'
      return 1
    fi
    echo "  ⚠ Retrying in 2s... (waiting for balance)"
    sleep 2
  done
}

echo "  Creating token account..."
retry_cmd spl-token create-account \
  --fee-payer $pwd/artifacts/spl_authority.json \
  --owner $pwd/artifacts/token_bank.json \
  -u $SOLANA_RPC_PROVIDER_URL \
  $MINT_ADDRESS || true

ATA_ADDRESS=$(spl-token accounts --owner $pwd/artifacts/token_bank.json $MINT_ADDRESS --addresses-only -u $SOLANA_RPC_PROVIDER_URL)
echo "  ATA:        $ATA_ADDRESS"

echo "  Minting 1,000,000 tokens..."
retry_cmd spl-token mint \
  --fee-payer $pwd/artifacts/spl_authority.json \
  --mint-authority $pwd/artifacts/spl_authority.json \
  -u $SOLANA_RPC_PROVIDER_URL \
  $MINT_ADDRESS 1000000000000 $ATA_ADDRESS

echo "  ✓ Tokens minted"
