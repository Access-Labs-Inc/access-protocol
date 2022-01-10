use crate::structs::api_response::ApiResponse;
use actix_web::{dev::HttpResponseBuilder, error, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum AccessError {
    #[display(fmt = "internal error")]
    InternalError,
    #[display(fmt = "bad request")]
    BadClientData,
    #[display(fmt = "invalid nonce")]
    InvalidNonce,
    #[display(fmt = "invalid pubkey")]
    InvalidPubkey,
    #[display(fmt = "invalid signed nonce")]
    InvalidSignedNonce,
    #[display(fmt = "invalid jwt")]
    InvalidJwt,
}

impl error::ResponseError for AccessError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code()).json(ApiResponse::new(false, self.to_string()))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            AccessError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AccessError::BadClientData => StatusCode::BAD_REQUEST,
            AccessError::InvalidNonce => StatusCode::BAD_REQUEST,
            AccessError::InvalidPubkey => StatusCode::BAD_REQUEST,
            AccessError::InvalidSignedNonce => StatusCode::BAD_REQUEST,
            AccessError::InvalidJwt => StatusCode::FORBIDDEN,
        }
    }
}
