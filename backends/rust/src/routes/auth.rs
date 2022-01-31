use crate::{
    errors::AccessError,
    structs::{
        api_response::ApiResponse,
        app_data::AppData,
        auth::{LoginRequest, LoginResponse, NonceRequest, NonceResponse},
    },
    utils::{
        jwt::create_jwt,
        nonce::generate_nonce,
        pubkey::is_valid_pubkey,
        request::{deserialize_body, load_body},
        settings::REDIS_EXPIRE,
        stake::check_stake_account,
    },
};
use {
    actix_web::{
        post,
        web::{Data, Payload},
        Error, HttpResponse,
    },
    hex,
    nacl::sign::verify,
    redis::Commands,
    solana_program::pubkey::Pubkey,
    std::str::FromStr,
    std::sync::Arc,
};

/**
 * Generates a nonce for a user and stores it in redis
 */
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

/**
 * Verifies the signed nonce returned by the user and returns a JWT
 */
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

    let staker_key = Pubkey::from_str(address.as_str()).map_err(|_| AccessError::InvalidPubkey)?;

    check_stake_account(staker_key).await?;

    // Create JWT
    let jwt = create_jwt(address)?;

    Ok(HttpResponse::Ok().json(ApiResponse::new(true, LoginResponse { token: jwt })))
}
