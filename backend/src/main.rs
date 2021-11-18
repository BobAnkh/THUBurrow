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
    match log4rs::init_file("/etc/backend/conf/log4rs.yml", Default::default()) {
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
        .attach(AdHoc::try_on_ignite("setup_users", setup::user_table_setup))
        .attach(AdHoc::try_on_ignite("setup_burrows", setup::burrow_table_setup))
        .attach(AdHoc::try_on_ignite("setup_junctions", setup::junction_table_setup))
}
