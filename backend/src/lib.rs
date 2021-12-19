#[macro_use]
extern crate rocket;

pub mod config;
pub mod db;
pub mod models;
pub mod pgdb;
pub mod pool;
pub mod routes;
pub mod setup;
pub mod utils;

use rocket::{fairing::AdHoc, Build, Rocket};
use rocket_db_pools::Database;

#[cfg(debug_assertions)]
pub fn log_init() {}

#[cfg(not(debug_assertions))]
pub fn log_init() {
    let filename = if std::path::Path::new("/etc/backend/conf/log4rs.yml").exists() {
        "/etc/backend/conf/log4rs.yml"
    } else if std::path::Path::new("/etc/backend/conf/log4rs-default.yml").exists() {
        "/etc/backend/conf/log4rs-default.yml"
    } else {
        "conf/log4rs.yml"
    };
    match log4rs::init_file(filename, Default::default()) {
        Ok(_) => (),
        Err(e) => panic!("Error initial logger: {}", e),
    }
}

pub fn rocket_init() -> Rocket<Build> {
    log_init();
    utils::id_gen::init(1);
    let cors_handler = utils::cors::init();
    rocket::build()
        .mount("/", rocket_cors::catch_all_options_routes())
        .attach(cors_handler.clone())
        .manage(cors_handler)
        .attach(pool::PgDb::init())
        .attach(pool::RedisDb::init())
        .attach(pool::PulsarSearchProducerMq::init())
        .attach(pool::MinioImageStorage::init())
        .attach(pool::TypesenseSearch::init())
        .attach(AdHoc::on_ignite("mount_routes", routes::routes_init))
        .attach(AdHoc::try_on_ignite(
            "setup_postgresql_tables",
            setup::postgres_table_setup,
        ))
}
