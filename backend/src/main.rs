#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

use backend::cors;
use backend::pool::{MinioImageStorage, PgDb, RedisDb};
use backend::routes::{self, sample, storage};
use backend::utils::id_gen;

#[cfg(debug_assertions)]
fn log_init() {}

#[cfg(not(debug_assertions))]
fn log_init() {
    match log4rs::init_file("conf/log4rs.yml", Default::default()) {
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
        .attach(PgDb::init())
        .attach(RedisDb::init())
        .attach(MinioImageStorage::init())
        .attach(cors_handler)
        .attach(AdHoc::on_ignite("mount_routes", routes::routes_init))
        .attach(AdHoc::on_ignite("mount_user", sample::init))
        .attach(AdHoc::on_ignite("mount_storage", storage::init))
}
