#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

mod cors;
mod db;
mod pool;
mod req;
mod routes;
mod utils;

use pool::{PgDb, RedisDb};
use routes::sample;
use routes::user_login;
use routes::user_signup;
use utils::id_gen;

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
