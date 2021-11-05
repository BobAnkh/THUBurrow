pub mod health;
pub mod sample;
pub mod storage;

use rocket::{fairing::AdHoc, Build, Rocket};

pub async fn routes_init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.attach(AdHoc::on_ignite("mount_health_check", health::init))
}
