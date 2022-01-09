use solana_program::pubkey::Pubkey;
use std::str::FromStr;

pub fn is_valid_pubkey(address: &str) -> bool {
    match Pubkey::from_str(address) {
        Ok(_) => true,
        _ => false,
    }
}
