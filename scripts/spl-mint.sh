#!/bin/bash
set -x
set -o allexport -o notify

pwd=$(pwd)
SOLANA_RPC_PROVIDER_URL=https://api.devnet.solana.com
MINT_ADDRESS=$(cat $pwd/artifacts/mint_address.txt)

solana-keygen new --outfile $pwd/artifacts/token_bank.json --no-bip39-passphrase
PUBKEY=$(solana-keygen pubkey $pwd/artifacts/token_bank.json)
echo "Token bank pubkey: $PUBKEY"
echo "Mint address: $MINT_ADDRESS"
spl-token create-account --fee-payer $pwd/artifacts/spl_authority.json --owner $pwd/artifacts/token_bank.json -u $SOLANA_RPC_PROVIDER_URL $MINT_ADDRESS
ATA_ADDRESS=$(spl-token accounts -v --owner $pwd/artifacts/token_bank.json | grep $MINT_ADDRESS | awk '{print $3}')
spl-token mint --fee-payer $pwd/artifacts/spl_authority.json --mint-authority $pwd/artifacts/spl_authority.json -u $SOLANA_RPC_PROVIDER_URL $MINT_ADDRESS 1000000000000 $ATA_ADDRESS