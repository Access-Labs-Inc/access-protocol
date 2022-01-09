use hex;
use rand::Rng;

pub fn generate_nonce() -> String {
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    hex::encode(random_bytes)
}
