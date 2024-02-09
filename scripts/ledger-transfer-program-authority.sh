#!/bin/bash

# Change these variables
LEDGER_ACCOUNT_INDEX=0
PROGRAM_PUBKEY=
SOLANA_RPC_PROVIDER_URL=
NEW_AUTHORITY_ADDRESS=

# Don't change anything below this line
LEDGER_KEYPAIR_URL=usb://ledger\?key=$LEDGER_ACCOUNT_INDEX
LEDGER_PUBKEY=$(solana-keygen pubkey "$LEDGER_KEYPAIR_URL")
if [ $? -eq 0 ]; then
    echo Ledger pubkey: $LEDGER_PUBKEY
else
    echo "Is your Ledger connected and unlocked?"
    exit 1
fi

CURRENT_AUTHORITY_ADDRESS=$(solana program show --url $SOLANA_RPC_PROVIDER_URL $PROGRAM_PUBKEY | grep "Authority" | awk '{print $2}')
if [ "$LEDGER_PUBKEY" != "$CURRENT_AUTHORITY_ADDRESS" ]; then
  echo "The Ledger pubkey $LEDGER_PUBKEY does not match the current authority $CURRENT_AUTHORITY_ADDRESS"
  exit 1
fi

echo "Transferring authority out of the Ledger..."
solana program set-upgrade-authority $PROGRAM_PUBKEY \
  --new-upgrade-authority $NEW_AUTHORITY_ADDRESS \
  -k "$LEDGER_KEYPAIR_URL" \
  --url $SOLANA_RPC_PROVIDER_URL \
  --skip-new-upgrade-authority-signer-check
