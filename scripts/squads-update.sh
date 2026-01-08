#!/bin/bash

# Change these variables
BUFFER_AUTHORITY_KEYPAIR=
PROGRAM_FILEPATH=
SOLANA_RPC_PROVIDER_URL=

# Don't change anything below this line

BUFFER_ADDRESS=$(solana program write-buffer $PROGRAM_FILEPATH \
  --url $SOLANA_RPC_PROVIDER_URL \
  -k $BUFFER_AUTHORITY_KEYPAIR | grep "Buffer" | awk '{print $2}')
echo "Buffer address: $BUFFER_ADDRESS"
echo "Buffer refund:" $(solana-keygen pubkey $BUFFER_AUTHORITY_KEYPAIR)

echo "Now finish the first step of the update in squads"
echo "and input the squads update authority address: "
read -r SQUADS_UPDATE_AUTHORITY_ADDRESS

solana program set-buffer-authority "$BUFFER_ADDRESS" \
  --new-buffer-authority "$SQUADS_UPDATE_AUTHORITY_ADDRESS" \
  -k $BUFFER_AUTHORITY_KEYPAIR \
  --url $SOLANA_RPC_PROVIDER_URL