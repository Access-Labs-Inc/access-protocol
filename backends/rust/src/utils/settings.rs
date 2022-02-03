use {dotenv, lazy_static::lazy_static};

/*
    Redis settings
*/

pub const REDIS_EXPIRE: usize = 60 * 10;

/*
    JWT settings
*/
pub const JWT_EXPIRE: u64 = 24 * 60 * 60;

lazy_static! {
    pub static ref JWT_ACCESS_TOKEN: String = dotenv::var("ACCESS_TOKEN_SECRET").unwrap();
    pub static ref REDIS_URL: String = dotenv::var("REDIS_URL").unwrap();
}
