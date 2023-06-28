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
    let mint = tr.mint_subscription_nft(&owner,
                                        "test".to_string(),
                                        "TEST".to_string(),
                                        "https://arweave.net/8jXYBs1Ddf97vVH82W1VElyIQ1AXog2Eh-78hdwX3a4".to_string(),
    ).await.unwrap();
    println!("minted subscription nft {:?} to {:?}, secret: {:?}",
            mint,
             owner.pubkey(),
             owner.to_bytes(),
    );
}