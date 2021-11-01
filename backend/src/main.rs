#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

use backend::cors;
use backend::pool::{PgDb, RedisDb};
use backend::routes::sample;
use backend::utils::id_gen;

#[launch]
fn rocket() -> _ {
    id_gen::init(1);
    let cors_handler = cors::init();
    rocket::build()
        .attach(PgDb::init())
        .attach(RedisDb::init())
        .attach(cors_handler)
        .attach(AdHoc::on_ignite("mount_user", sample::init))
        .attach(AdHoc::on_ignite("mount_user", user_signup::init))
        .attach(AdHoc::on_ignite("mount_user", user_login::init))
}
