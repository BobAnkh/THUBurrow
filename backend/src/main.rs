#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

mod pool;
mod routes;
mod req;
mod db;

use pool::{PgDb, RedisDb};
use routes::sample;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(PgDb::init())
        .attach(RedisDb::init())
        .attach(AdHoc::on_ignite("mount_user", sample::init))
}
