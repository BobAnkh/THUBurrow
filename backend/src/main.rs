#[macro_use]
extern crate rocket;

use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use rocket_db_pools::Database;

use backend::cors;
use backend::pool::{MinioImageStorage, PgDb, RedisDb};
use backend::routes::{self, sample};
use backend::setup;
use backend::utils::id_gen;

#[cfg(debug_assertions)]
fn log_init() {}

#[cfg(not(debug_assertions))]
fn log_init() {
    match log4rs::init_file("/etc/backend/conf/log4rs.yml", Default::default()) {
        Ok(_) => (),
        Err(e) => panic!("Error initial logger: {}", e),
    }
}

async fn user_table_setup(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &PgDb::fetch(&rocket).unwrap().connection;
    let _ = setup::create_user_table(conn).await;
    Ok(rocket)
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
        .attach(AdHoc::try_on_ignite("Migrations", user_table_setup))
}
