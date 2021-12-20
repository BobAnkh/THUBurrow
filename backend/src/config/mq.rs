use lazy_static::lazy_static;
use std::collections::HashMap;

pub static EMAIL_TOKEN_EX: i32 = 14400;

lazy_static! {
    pub static ref TYPESENSE_API_KEY: String = {
        let env_v = "ROCKET_DATABASES=".to_string() + &std::env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| r#"{search={url="http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="}}"#.to_string());
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("search") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="
                        .to_string(),
                },
                None => {
                    "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
                }
            },
            None => {
                "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
            }
        };
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
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("search") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24="
                        .to_string(),
                },
                None => {
                    "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
                }
            },
            None => {
                "http://127.0.0.1:8108@8Dz4jRrsBjYgdCD/VGP1bleph7oBThJr5IcF43l0U24=".to_string()
            }
        };
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
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("pgdb") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "postgres://postgres:postgres@127.0.0.1:5432/pgdb".to_string(),
                },
                None => "postgres://postgres:postgres@127.0.0.1:5432/pgdb".to_string(),
            },
            None => "postgres://postgres:postgres@127.0.0.1:5432/pgdb".to_string(),
        };
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
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("pulsar-mq") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "pulsar://127.0.0.1:6650".to_string(),
                },
                None => "pulsar://127.0.0.1:6650".to_string(),
            },
            None => "pulsar://127.0.0.1:6650".to_string(),
        };
        url
    };
    pub static ref REDIS_ADDR: String = {
        let env_v = "ROCKET_DATABASES=".to_string()
            + &std::env::var("ROCKET_DATABASES").ok().unwrap_or_else(|| {
                r#"{keydb={url="redis://:keypassword@127.0.0.1:6300"}}"#.to_string()
            });
        let env_v =
            toml::from_str::<HashMap<String, HashMap<String, HashMap<String, String>>>>(&env_v)
                .unwrap();
        let url: String = match env_v.get("ROCKET_DATABASES") {
            Some(r) => match r.get("keydb") {
                Some(r) => match r.get("url") {
                    Some(r) => r.to_owned(),
                    None => "redis://:keypassword@127.0.0.1:6300".to_string(),
                },
                None => "redis://:keypassword@127.0.0.1:6300".to_string(),
            },
            None => "redis://:keypassword@127.0.0.1:6300".to_string(),
        };
        url
    };
}
