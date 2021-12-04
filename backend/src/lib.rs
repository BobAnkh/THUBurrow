#[macro_use]
extern crate rocket;

pub mod db;
pub mod pgdb;
pub mod pool;
pub mod req;
pub mod routes;
pub mod setup;
pub mod utils;

use rocket::{fairing::AdHoc, Build, Rocket};
use rocket_db_pools::Database;

#[cfg(debug_assertions)]
fn log_init() {}

#[cfg(not(debug_assertions))]
fn log_init() {
    let filename = if std::path::Path::new("/etc/backend/conf/log4rs.yml").exists() {
        "/etc/backend/conf/log4rs.yml"
    } else {
        "/etc/backend/conf/log4rs-default.yml"
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
    let _ = vec![tokio::spawn(utils::mq::generate_trending())];
    let _ = vec![
        tokio::spawn(utils::mq::pulsar_relation()),
        tokio::spawn(utils::mq::pulsar_typesense()),
    ];
    rocket::build()
        .attach(cors_handler)
        .attach(pool::PgDb::init())
        .attach(pool::RedisDb::init())
        .attach(pool::PulsarSearchProducerMq::init())
        .attach(pool::MinioImageStorage::init())
        .attach(AdHoc::on_ignite("mount_sample", routes::sample::init))
        .attach(AdHoc::on_ignite("mount_routes", routes::routes_init))
        .attach(AdHoc::try_on_ignite(
            "setup_postgresql_tables",
            setup::postgres_table_setup,
        ))
}
