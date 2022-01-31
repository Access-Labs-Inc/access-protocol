use {solana_program::pubkey::Pubkey, std::str::FromStr};

/// Verifies the validity of a public key
pub fn is_valid_pubkey(address: &str) -> bool {
    Pubkey::from_str(address).is_ok()
}
