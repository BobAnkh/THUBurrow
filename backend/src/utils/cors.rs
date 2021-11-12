use rocket_cors::{AllowedOrigins, Cors};

pub fn init() -> Cors {
    let allowed_origins = AllowedOrigins::some(
        &["https://thuburrow.com"],
        &["^https://(.+).thuburrow.com$"],
    );

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allow_credentials: true,
        max_age: Some(3600),
        ..Default::default()
    }
    .to_cors();
    match cors {
        Ok(c) => c,
        _ => panic!("Can not initialize cors"),
    }
}
