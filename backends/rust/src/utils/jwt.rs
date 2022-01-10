use crate::errors::AccessError;
use actix_web::HttpRequest;
use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::time::SystemTime;

pub const EXPIRATION_INTERVAL: u64 = 24 * 60 * 60;
pub const SECRET: &'static [u8] = b"some secret";

pub fn create_jwt(address: String) -> Result<String, AccessError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(SECRET).map_err(|_| AccessError::InternalError)?;
    let mut claims = BTreeMap::new();
    claims.insert("address", address);
    claims.insert("iat", current_time().to_string());
    let jwt = claims
        .sign_with_key(&key)
        .map_err(|_| AccessError::InternalError)?;
    Ok(jwt)
}

pub fn verify_jwt(jwt: &str) -> Result<(), AccessError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(SECRET).map_err(|_| AccessError::InvalidJwt)?;
    let token: Token<Header, BTreeMap<String, String>, _> =
        VerifyWithKey::verify_with_key(jwt, &key).map_err(|_| AccessError::InvalidJwt)?;
    let claims = token.claims();
    let iat = claims["iat"]
        .parse::<u64>()
        .map_err(|_| AccessError::InvalidJwt)?;

    let now = current_time();
    if now - iat > EXPIRATION_INTERVAL {
        return Err(AccessError::InvalidJwt);
    }

    Ok(())
}

pub fn get_token_from_header(req: &HttpRequest) -> Result<&str, AccessError> {
    match req
        .headers()
        .get("authorization")
        .ok_or(AccessError::BadClientData)?
        .to_str()
    {
        Ok(header) => header.split(' ').last().ok_or(AccessError::BadClientData),
        _ => Err(AccessError::BadClientData),
    }
}

pub fn current_time() -> u64 {
    let now = SystemTime::now();
    now.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
