#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

mod cors;
mod db;
mod pool;
mod req;
mod routes;

use pool::Db;
use routes::sample;

#[launch]
fn rocket() -> _ {
    let cors_handler = cors::init();
    rocket::build()
        .attach(cors_handler)
        .attach(Db::init())
        .attach(AdHoc::on_ignite("mount_user", sample::init))
}
