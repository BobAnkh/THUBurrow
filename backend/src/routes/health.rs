//! Routes for health checks

use rocket::{Build, Rocket};

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/", routes![health_check])
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
