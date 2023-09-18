use solana_sdk::signer::Signer;
use access_protocol::instruction::ProgramInstruction::{AdminMint, ClaimRewards};
use access_protocol::utils::{get_freeze_mask, get_unfreeze_mask};
use crate::common::test_runner::TestRunner;

pub mod common;

#[tokio::test]
async fn program_freeze() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();
    tr.migrate_v2().await.unwrap();

    // Renounce a specific instruction
    let freeze_mask = get_freeze_mask(vec![access_protocol::instruction::ProgramInstruction::AdminMint]);
    println!("freeze mask: {:0128b}", freeze_mask);
    tr.renounce(AdminMint).await.unwrap();
    tr.sleep(1).await.unwrap();
    let staker = tr.create_user_with_ata().await.unwrap();
    tr.mint(&staker.pubkey(), 729_999_999_999).await.unwrap_err();

    // Renounce an instruction that is not renouncable
    let freeze_mask = get_freeze_mask(vec![access_protocol::instruction::ProgramInstruction::AdminMint]);
    println!("freeze mask: {:0128b}", freeze_mask);
    tr.renounce(ClaimRewards).await.unwrap_err();
    tr.sleep(1).await.unwrap();

    // Renounce a instruction again
    let freeze_mask = get_freeze_mask(vec![access_protocol::instruction::ProgramInstruction::AdminMint]);
    println!("freeze mask: {:0128b}", freeze_mask);
    tr.renounce(AdminMint).await.unwrap_err();
}