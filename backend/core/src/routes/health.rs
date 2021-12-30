//! Routes for health checks

use rocket::{Build, Rocket};

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/", routes![health_check, health_content, health_burrow,])
}

/// Health check
///
/// ## Returns
///
/// - `Status`: "Ok"
#[get("/health")]
async fn health_check() -> String {
    "Ok".to_string()
}

#[get("/content/posts/undefined")]
async fn health_content() -> String {
    "Ok".to_string()
}

#[get("/burrows/undefined")]
async fn health_burrow() -> String {
    "Ok".to_string()
}
