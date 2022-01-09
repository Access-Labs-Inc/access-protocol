use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    result: T,
}

impl<T> ApiResponse<T> {
    pub fn new(success: bool, data: T) -> ApiResponse<T> {
        Self {
            success,
            result: data,
        }
    }
}
