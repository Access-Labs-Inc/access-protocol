use redis::Client;

pub struct AppData {
    pub redis_client: Client,
}

impl AppData {
    pub fn new() -> AppData {
        Self {
            redis_client: Client::open("redis://127.0.0.1/").unwrap(),
        }
    }
}
