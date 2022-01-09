use crate::structs::{api_response::ApiResponse, app_data::AppData};
use crate::utils::nonce::generate_nonce;
use actix_web::{
    post,
    web::{Data, Payload},
    Error, HttpResponse,
};
use hex;
use nacl::sign::verify;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

use crate::utils::{
    pubkey::is_valid_pubkey,
    request::{deserialize_body, load_body},
};
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct NonceResponse {
    nonce: String,
}

#[derive(Serialize, Deserialize)]
pub struct NonceRequest {
    address: String,
}

#[post("/auth/nonce")]
pub async fn handle_get_nonce(
    data: Data<Arc<AppData>>,
    payload: Payload,
) -> Result<HttpResponse, Error> {
    let nonce = generate_nonce();

    let NonceRequest { address } = deserialize_body::<NonceRequest>(&load_body(payload).await);

    let is_valid_body = is_valid_pubkey(address.as_str());

    println!("Valid pubkey {}", is_valid_body);

    // Add nonce to Redis cache
    let mut connection = data.redis_client.get_connection().unwrap();
    let _: () = connection
        .set(format!("nonce:{}", &address), &nonce)
        .unwrap();

    let result = NonceResponse { nonce };
    Ok(HttpResponse::Ok().json(ApiResponse::new(true, result)))
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    address: String,
    #[serde(rename = "signedNonce")]
    signed_nonce: String,
}

#[post("/auth/login")]
pub async fn handle_login(
    data: Data<Arc<AppData>>,
    payload: Payload,
) -> Result<HttpResponse, Error> {
    let LoginRequest {
        address,
        signed_nonce,
    } = deserialize_body::<LoginRequest>(&load_body(payload).await);

    let mut connection = data.redis_client.get_connection().unwrap();
    let nonce: String = connection.get(format!("nonce:{}", &address)).unwrap();

    let is_valid_nonce = verify(
        hex::decode(signed_nonce).unwrap().as_slice(),
        nonce.as_bytes(),
        &Pubkey::from_str(&address).unwrap().to_bytes(),
    )
    .unwrap();

    println!("is_valid_nonce {}", is_valid_nonce);

    let result = "";
    Ok(HttpResponse::Ok().json(ApiResponse::new(true, result)))
}
