use rocket_cors::{AllowedOrigins, Cors};

pub fn init() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "https://api.thuburrow.com",
        "https://thuburrow.com",
        "https://search.thurrow.com",
        "https://static.thuburrow.com",
    ]);

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allow_credentials: true,
        max_age: Some(1728000),
        ..Default::default()
    }
    .to_cors();
    match cors {
        Ok(c) => c,
        _ => panic!("Can not initialize cors"),
    }
}
