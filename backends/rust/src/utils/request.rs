use crate::errors::AccessError;
use {
    actix_web::web::{BytesMut, Payload},
    futures::StreamExt,
    serde::Deserialize,
};

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// Loads the body of an incoming request
pub async fn load_body(mut payload: Payload) -> Result<BytesMut, AccessError> {
    // payload is a stream of Bytes objects
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|_| AccessError::InternalError)?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            // return Err(error::ErrorBadRequest("overflow"));
            panic!()
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// Deserializes a request body in a struct T
pub fn deserialize_body<'a, T: Deserialize<'a>>(bytes: &'a BytesMut) -> T {
    serde_json::from_slice::<T>(bytes).unwrap()
}
