//! Module of configuration

pub mod burrow;
pub mod content;
pub mod email;
pub mod mq;
pub mod storage;
pub mod user;

lazy_static::lazy_static! {
    pub static ref BACKEND_TEST_MODE: bool = std::env::var("BACKEND_TEST_MODE")
        .map(|x| x.parse::<bool>().unwrap_or(false))
        .ok()
        .unwrap_or(false);
}
