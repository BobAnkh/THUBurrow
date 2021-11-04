use rocket::{Build, Rocket};

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/",
        routes![
            health_check
        ],
    )
}

#[get("/health")]
async fn health_check() -> String {
    "Ok".to_string()
}