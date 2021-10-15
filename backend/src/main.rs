#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

mod pool;
mod routes;
mod req;
mod db;

use pool::Db;
use routes::sample;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::on_ignite("mount_user", sample::init))
}
