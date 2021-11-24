use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Params {
    // Amount to stake
    pub amount: u64,
}
