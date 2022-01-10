use crate::errors::AccessError;
use crate::structs::api_response::ApiResponse;
use crate::utils::jwt::{get_token_from_header, verify_jwt};
use actix_web::{get, Error, HttpRequest, HttpResponse};

#[get("/article")]
pub async fn handle_get_article(req: HttpRequest) -> Result<HttpResponse, Error> {
    let jwt = get_token_from_header(&req).map_err(|_| AccessError::BadClientData)?;

    verify_jwt(jwt)?;

    Ok(HttpResponse::Ok().json(ApiResponse::new(true, "Hello world")))
}
