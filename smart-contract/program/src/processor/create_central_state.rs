use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Params {
    // Nonce of the central state PDA
    pub signer_nonce: u8,
}
