//! Module for routes

pub mod admin;
pub mod burrow;
pub mod content;
pub mod health;
pub mod search;
pub mod storage;
pub mod trending;
pub mod user;

use rocket::{fairing::AdHoc, Build, Rocket};

/// Attach routes to rocket
pub async fn routes_init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .attach(AdHoc::on_ignite("mount_health_check", health::init))
        .attach(AdHoc::on_ignite("mount_content", content::init))
        .attach(AdHoc::on_ignite("mount_user", user::init))
        .attach(AdHoc::on_ignite("mount_storage", storage::init))
        .attach(AdHoc::on_ignite("mount_search", search::init))
        .attach(AdHoc::on_ignite("mount_burrow", burrow::init))
        .attach(AdHoc::on_ignite("mount_trending", trending::init))
        .attach(AdHoc::on_ignite("mount_admin", admin::init))
}
