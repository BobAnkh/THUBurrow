use lazy_static::lazy_static;

lazy_static! {
    pub static ref SECRET_ID: String = std::env::var("SECRET_ID").ok().unwrap_or_else(|| "".to_string());
    pub static ref SECRET_KEY: String = std::env::var("SECRET_KEY")
        .ok()
        .unwrap_or_else(|| "".to_string());
}
