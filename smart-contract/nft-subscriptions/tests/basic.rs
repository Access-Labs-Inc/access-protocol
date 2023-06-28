use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_test_framework::*;

use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn basic() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new().await.unwrap();
    let owner = Keypair::new();
    let mint = tr.mint_subscription_nft(&owner).await.unwrap();
    println!("minted subscription nft {:?} to {:?}, secret: {:?}",
            mint,
             owner.pubkey(),
             owner.to_bytes(),
    );
}