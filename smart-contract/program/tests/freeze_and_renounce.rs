use solana_sdk::signature::{Keypair, Signer};
use access_protocol::instruction::ProgramInstruction::AdminProgramFreeze;
use access_protocol::utils::{get_freeze_mask, get_unfreeze_mask};
use crate::common::test_runner::TestRunner;

mod common;

#[tokio::test]
async fn program_freeze() {
    // Setup the token + basic accounts
    let mut tr = TestRunner::new(1_000_000).await.unwrap();

    // Create a fee authority account
    let fee_authority = Keypair::new();
    tr.get_sol(&fee_authority.pubkey(), 100_000_000)
        .await
        .unwrap();

    // Freeze the program
    let freeze_mask = get_freeze_mask(vec![]);
    println!("freeze mask: {:0128b}", freeze_mask);
    let staker = tr.create_user_with_ata().await.unwrap();
    tr.freeze_program(freeze_mask, None).await.unwrap();
    tr.sleep(1).await.unwrap();
    tr.mint(&staker.pubkey(), 729_999_999_999).await.unwrap_err();

    // Unfreeze the program
    let unfreeze_mask = get_unfreeze_mask(vec![]);
    println!("unfreeze mask: {:0128b}", unfreeze_mask);
    tr.freeze_program(unfreeze_mask, None).await.unwrap();
    tr.sleep(1).await.unwrap();
    tr.mint(&staker.pubkey(), 729_999_999_999).await.unwrap();

    // Try freezing with different authority
    tr.freeze_program(freeze_mask, Some(&fee_authority)).await.unwrap_err();

    // Set the authority
    tr.change_freeze_authority(&fee_authority).await.unwrap();
    tr.sleep(1).await.unwrap();

    // Partial freeze with the new authority should fail
    tr.freeze_program(1, Some(&fee_authority)).await.unwrap_err();

    // Full freeze with the new authority should work
    tr.freeze_program(0, Some(&fee_authority)).await.unwrap();

    // Freeze a specific instruction
    let freeze_mask = get_freeze_mask(vec![access_protocol::instruction::ProgramInstruction::AdminMint]);
    println!("freeze mask: {:0128b}", freeze_mask);
    tr.freeze_program(freeze_mask, None).await.unwrap();
    tr.sleep(1).await.unwrap();
    tr.mint(&staker.pubkey(), 729_999_999_999).await.unwrap_err();
    tr.change_inflation(1_100_000_000).await.unwrap();

    // Unfreeze a specific instruction
    let unfreeze_mask = get_unfreeze_mask(vec![access_protocol::instruction::ProgramInstruction::AdminMint]);
    println!("unfreeze mask: {:0128b}", unfreeze_mask);
    tr.freeze_program(unfreeze_mask, None).await.unwrap();
    tr.sleep(1).await.unwrap();
    tr.mint(&staker.pubkey(), 729_999_999_999).await.unwrap();
    tr.change_inflation(1_100_000_000).await.unwrap_err();

    // Freeze a specific instruction
    let freeze_mask = get_freeze_mask(vec![access_protocol::instruction::ProgramInstruction::AdminMint]);
    println!("freeze mask: {:0128b}", freeze_mask);
    tr.freeze_program(freeze_mask, None).await.unwrap();
    tr.sleep(1).await.unwrap();
    tr.mint(&staker.pubkey(), 729_999_999_999).await.unwrap_err();
    tr.change_inflation(1_100_000_000).await.unwrap();

    // Renounce freeze functionality
    tr.renounce(AdminProgramFreeze).await.unwrap();
    tr.sleep(1).await.unwrap();

    // Unfreeze should not work anymore
    let unfreeze_mask = get_unfreeze_mask(vec![]);
    println!("unfreeze mask: {:0128b}", unfreeze_mask);
    tr.freeze_program(unfreeze_mask, None).await.unwrap_err();
}