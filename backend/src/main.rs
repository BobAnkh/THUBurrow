#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

use backend::pool::{MinioImageStorage, PgDb, RedisDb};
use backend::routes::{self, sample};
use backend::setup;
use backend::utils::{cors, id_gen};

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

#[launch]
fn rocket() -> _ {
    log_init();
    id_gen::init(1);
    let cors_handler = cors::init();
    rocket::build()
        .attach(cors_handler)
        .attach(PgDb::init())
        .attach(RedisDb::init())
        .attach(MinioImageStorage::init())
        .attach(AdHoc::on_ignite("mount_user", sample::init))
        .attach(AdHoc::on_ignite("mount_routes", routes::routes_init))
        .attach(AdHoc::try_on_ignite(
            "Migrations",
            setup::postgres_table_setup,
        ))
}
