pub mod burrow;
pub mod health;
pub mod sample;
pub mod storage;
pub mod user;

use rocket::{fairing::AdHoc, Build, Rocket};

pub async fn routes_init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .attach(AdHoc::on_ignite("mount_health_check", health::init))
        .attach(AdHoc::on_ignite("mount_user", user::init))
        .attach(AdHoc::on_ignite("mount_storage", storage::init))
        .attach(AdHoc::on_ignite("mount_burrow", burrow::init))
}
