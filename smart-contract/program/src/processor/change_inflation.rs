use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Params {
    // The new inflation rate
    pub inflation_rate: u64,
}
