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

use crate::errors::AccessError;
use crate::structs::auth::{LoginRequest, LoginResponse, NonceRequest, NonceResponse};
use crate::utils::settings::REDIS_EXPIRE;
use crate::utils::{
    jwt::create_jwt,
    pubkey::is_valid_pubkey,
    request::{deserialize_body, load_body},
};
use redis::Commands;
use std::sync::Arc;

#[post("/auth/nonce")]
pub async fn handle_get_nonce(
    data: Data<Arc<AppData>>,
    payload: Payload,
) -> Result<HttpResponse, Error> {
    let nonce = generate_nonce();

    let NonceRequest { address } = deserialize_body::<NonceRequest>(&load_body(payload).await?);

    let is_valid_body = is_valid_pubkey(address.as_str());

    if !is_valid_body {
        return Err(AccessError::BadClientData.into());
    }

    // Add nonce to Redis cache
    let mut connection = data
        .redis_client
        .get_connection()
        .map_err(|_| AccessError::InternalError)?;

    let _: () = connection
        .set_ex(format!("nonce:{}", &address), &nonce, REDIS_EXPIRE)
        .map_err(|_| AccessError::InternalError)?;

    let result = NonceResponse { nonce };
    Ok(HttpResponse::Ok().json(ApiResponse::new(true, result)))
}

#[post("/auth/login")]
pub async fn handle_login(
    data: Data<Arc<AppData>>,
    payload: Payload,
) -> Result<HttpResponse, Error> {
    let LoginRequest {
        address,
        signed_nonce,
    } = deserialize_body::<LoginRequest>(&load_body(payload).await?);

    let mut connection = data
        .redis_client
        .get_connection()
        .map_err(|_| AccessError::InternalError)?;

    let nonce: String = connection
        .get(format!("nonce:{}", &address))
        .map_err(|_| AccessError::InvalidNonce)?;

    verify(
        hex::decode(signed_nonce)
            .map_err(|_| AccessError::InvalidSignedNonce)?
            .as_slice(),
        nonce.as_bytes(),
        &Pubkey::from_str(&address)
            .map_err(|_| AccessError::InvalidPubkey)?
            .to_bytes(),
    )
    .map_err(|_| AccessError::InvalidNonce)?;

    // TODO check staked amount

    // Create JWT
    let jwt = create_jwt(address)?;

    Ok(HttpResponse::Ok().json(ApiResponse::new(true, LoginResponse { token: jwt })))
}
