#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

use backend::cors;
use backend::pool::{MinioImageStorage, PgDb, RedisDb};
use backend::routes::{sample, storage};
use backend::utils::id_gen;

#[launch]
fn rocket() -> _ {
    id_gen::init(1);
    let cors_handler = cors::init();
    rocket::build()
        .attach(PgDb::init())
        .attach(RedisDb::init())
        .attach(MinioImageStorage::init())
        .attach(cors_handler)
        .attach(AdHoc::on_ignite("mount_user", sample::init))
        .attach(AdHoc::on_ignite("mount_storage", storage::init))
}
