use solana_program::pubkey::Pubkey;
use std::str::FromStr;

pub fn is_valid_pubkey(address: &str) -> bool {
    Pubkey::from_str(address).is_ok()
}
