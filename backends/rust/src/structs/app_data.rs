use crate::utils::settings::REDIS_URL;
use redis::Client;

pub struct AppData {
    pub redis_client: Client,
}

impl AppData {
    pub fn new() -> AppData {
        Self {
            redis_client: Client::open(REDIS_URL.as_str()).unwrap(),
        }
    }
}
