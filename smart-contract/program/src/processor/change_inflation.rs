use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Params {
    // The new daily inflation token amount
    pub daily_inflation: u64,
}
