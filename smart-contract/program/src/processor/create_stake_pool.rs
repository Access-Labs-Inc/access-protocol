use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Params {
    // The PDA nonce
    pub nonce: u8,
}
