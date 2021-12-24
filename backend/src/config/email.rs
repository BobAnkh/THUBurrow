use lazy_static::lazy_static;

lazy_static! {
    pub static ref SECRET_ID: String = std::env::var("SECRET_ID")
        .ok()
        .unwrap_or_else(|| "".to_string());
    pub static ref SECRET_KEY: String = std::env::var("SECRET_KEY")
        .ok()
        .unwrap_or_else(|| "".to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_id_config() {
        assert_eq!("", *SECRET_ID);
    }

    #[test]
    fn test_secret_key_config() {
        assert_eq!("", *SECRET_KEY);
    }
}
