use lazy_static::lazy_static;
use std::collections::HashMap;

pub static EMAIL_TOKEN_EX: i32 = 14400;

lazy_static! {
    pub static ref TYPESENSE_API_KEY: String = {
        let env_v = "ROCKET_DATABASES=".to_string() + &std::env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| r#"{search={url="http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url = env_v
            .get("ROCKET_DATABASES")
            .and_then(|r| r.get("search"))
            .and_then(|r| r.get("url"))
            .cloned()
            .unwrap_or_else(|| {
                "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
            });
        let info: Vec<&str> = url.split('@').collect();
        let api_key: String;
        if info.len() == 1 {
            api_key = "8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_owned();
        } else if info.len() == 2 {
            api_key = info[1].to_owned();
        } else {
            panic!("Invalid typesense url.");
        }
        api_key
    };
    pub static ref TYPESENSE_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string() + &std::env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| r#"{search={url="http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url = env_v
            .get("ROCKET_DATABASES")
            .and_then(|r| r.get("search"))
            .and_then(|r| r.get("url"))
            .cloned()
            .unwrap_or_else(|| {
                "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
            });
        let info: Vec<&str> = url.split('@').collect();
        let addr: String;
        if info.len() == 1 || info.len() == 2 {
            addr = info[0].to_owned();
        } else {
            panic!("Invalid typesense url.");
        }
        addr
    };
    pub static ref POSTGRES_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string()
            + &std::env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| {
                r#"{pgdb={url="postgres://postgres:postgres@127.0.0.1:5432/pgdb"}}"#.to_string()
            });
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url = env_v
            .get("ROCKET_DATABASES")
            .and_then(|r| r.get("pgdb"))
            .and_then(|r| r.get("url"))
            .cloned()
            .unwrap_or_else(|| "postgres://postgres:postgres@127.0.0.1:5432/pgdb".to_string());
        url
    };
    pub static ref PULSAR_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string()
            + &std::env::var("ROCKET_DATABASES")
                .ok()
                .unwrap_or_else(|| r#"{pulsar-mq={url="pulsar://127.0.0.1:6650"}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url = env_v
            .get("ROCKET_DATABASES")
            .and_then(|r| r.get("pulsar-mq"))
            .and_then(|r| r.get("url"))
            .cloned()
            .unwrap_or_else(|| "pulsar://127.0.0.1:6650".to_string());
        url
    };
    pub static ref REDIS_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string()
            + &std::env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| {
                r#"{redis={url="redis://:keypassword@127.0.0.1:6300"}}"#.to_string()
            });
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url = env_v
            .get("ROCKET_DATABASES")
            .and_then(|r| r.get("redis"))
            .and_then(|r| r.get("url"))
            .cloned()
            .unwrap_or_else(|| "redis://:keypassword@127.0.0.1:6300".to_string());
        url
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typesense_api_key() {
        assert_eq!(
            *TYPESENSE_API_KEY,
            "8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
        );
    }

    #[test]
    fn test_typesense_addr() {
        assert_eq!(*TYPESENSE_ADDR, "http://127.0.0.1:8108".to_string());
    }

    #[test]
    fn test_postgres_addr() {
        assert_eq!(
            *POSTGRES_ADDR,
            "postgres://postgres:postgres@127.0.0.1:5432/pgdb".to_string()
        );
    }

    #[test]
    fn test_pulsar_addr() {
        assert_eq!(*PULSAR_ADDR, "pulsar://127.0.0.1:6650".to_string());
    }

    #[test]
    fn test_redis_addr() {
        assert_eq!(
            *REDIS_ADDR,
            "redis://:keypassword@127.0.0.1:6300".to_string()
        );
    }
}
