#!/bin/bash
# set -x

pwd=$(pwd)
NETWORK=${NETWORK:-devnet}

pushd ../smart-contract/program
echo  "Building smart contract program..."
# IMPORTANT: If not no-mint-check present you need to update MINT_ADDRESS in (state.rs)
cargo build-bpf --features no-bond-signer no-mint-check

PROGRAM_KEYPAIR=${PROGRAM_KEYPAIR:-"$pwd/program.json"}
echo "Check program keypair file exists..."
if test -f "$PROGRAM_KEYPAIR"
then
  echo "Program ID keypair exists at: $PROGRAM_KEYPAIR"
else
  echo "Creating program keypair..."
  solana-keygen new --outfile $PROGRAM_KEYPAIR
fi

AUTHORITY_KEYPAIR=${AUTHORITY_KEYPAIR:-"$pwd/authority.json"}
echo "Check fee payer keypair file exists..."
if test -f "$AUTHORITY_KEYPAIR"
then
  echo "Authority ID keypair exists at: $AUTHORITY_KEYPAIR"
else
  if [ "$NETWORK" == "devnet" ];
  then
    echo "Creating authority keypair..."
    solana-keygen new --outfile $AUTHORITY_KEYPAIR
  else
    echo "For production env. you need to provide the AUTHORITY_KEYPAIR with SOL inside!"
    exit 1
  fi
fi

solana config set --keypair $AUTHORITY_KEYPAIR

echo "authority: $(solana address)"

echo "Checking your account balance..."
balance=$(solana balance -u ${NETWORK} | rev | grep -Eo '[^ ]+$' | rev)
echo $balance
if (( $(echo "$balance > 6" | bc -l) ))
then
  echo "Balance is good."
else
  if [ "$NETWORK" == "devnet" ];
  then
    while [ ${balance%.*} -lt 4 ]
    do
      echo "Not enough SOL in your wallet, airdropping..."
      solana airdrop 1
      sleep 2
      balance=$(solana balance -u ${NETWORK} | rev | grep -Eo '[^ ]+$' | rev)
    done
  else
    echo "You need at least 6 SOL in the wallet to be able to deploy!"
    exit 1
  fi
fi

echo "Deploying contract..."
program_pubkey=$(solana-keygen pubkey ${PROGRAM_KEYPAIR})
echo "Program pubkey: $program_pubkey"
authority_pubkey=$(solana-keygen pubkey ${AUTHORITY_KEYPAIR})
echo "Authority pubkey: $authority_pubkey"
solana program deploy ./target/deploy/access_protocol.so \
 --program-id ${PROGRAM_KEYPAIR} \
 --upgrade-authority ${authority_pubkey} \
 -u ${NETWORK}

popd
