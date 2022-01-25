/*
    Redis settings
*/
pub const REDIS_URL: &str = "redis://127.0.0.1/";
pub const REDIS_EXPIRE: usize = 60 * 10;

/*
    JWT settings
*/
pub const JWT_EXPIRE: u64 = 24 * 60 * 60;
pub const JWT_ACCESS_TOKEN: &str = ""; // TODO use dotenv!
