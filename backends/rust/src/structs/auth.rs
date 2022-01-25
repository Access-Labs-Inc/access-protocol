use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: String,
}

#[derive(Serialize, Deserialize)]
pub struct NonceRequest {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub address: String,
    #[serde(rename = "signedNonce")]
    pub signed_nonce: String,
}
