#!/bin/bash
set -x

pwd=$(pwd)
export BPF_OUT_DIR=$pwd/../program/target/deploy

pushd ../../metaplex-program-library/token-metadata/program
echo "Building metaplex token metadata program..."
cargo build-sbf --sbf-out-dir $BPF_OUT_DIR --arch bpf
popd

cp $pwd/../program/target/deploy/mpl_token_metadata.so $pwd/programs/