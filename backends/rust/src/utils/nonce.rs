use hex;
use rand::Rng;

/// Generates a random 32 bytes nonce encoded as hex string
pub fn generate_nonce() -> String {
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    hex::encode(random_bytes)
}
