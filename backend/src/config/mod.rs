pub mod email;
pub mod mq;
pub mod user;

lazy_static::lazy_static! {
    pub static ref BACKEND_TEST_MODE: bool = std::env::var("BACKEND_TEST_MODE")
        .map(|x| x.parse::<bool>().unwrap_or(true))
        .ok()
        .unwrap_or(true);
}
