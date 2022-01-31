use crate::{
    errors::AccessError,
    utils::settings::{JWT_ACCESS_TOKEN, JWT_EXPIRE},
};
use {
    actix_web::HttpRequest,
    hmac::{Hmac, Mac},
    jwt::{Header, SignWithKey, Token, VerifyWithKey},
    sha2::Sha256,
    std::{collections::BTreeMap, time::SystemTime},
};

/// Create a new JWT token with two claims
/// - `address`
/// - `iat`
pub fn create_jwt(address: String) -> Result<String, AccessError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(JWT_ACCESS_TOKEN.as_bytes())
        .map_err(|_| AccessError::InternalError)?;
    let mut claims = BTreeMap::new();
    claims.insert("address", address);
    claims.insert("iat", current_time().to_string());
    let jwt = claims
        .sign_with_key(&key)
        .map_err(|_| AccessError::InternalError)?;
    Ok(jwt)
}

/// Verifies a JWT
pub fn verify_jwt(jwt: &str) -> Result<(), AccessError> {
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(JWT_ACCESS_TOKEN.as_bytes()).map_err(|_| AccessError::InvalidJwt)?;
    let token: Token<Header, BTreeMap<String, String>, _> =
        VerifyWithKey::verify_with_key(jwt, &key).map_err(|_| AccessError::InvalidJwt)?;
    let claims = token.claims();
    let iat = claims["iat"]
        .parse::<u64>()
        .map_err(|_| AccessError::InvalidJwt)?;

    let now = current_time();
    if now - iat > JWT_EXPIRE {
        return Err(AccessError::InvalidJwt);
    }

    Ok(())
}

/// Extract the bearer token from the `authorization` header
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

/// Returns the current unix timestamp in seconds
pub fn current_time() -> u64 {
    let now = SystemTime::now();
    now.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
