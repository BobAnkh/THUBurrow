#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

mod pool;
mod routes;
mod req;
mod db;
mod utils;

use pool::Db;
use routes::sample;
use utils::id_gen;

#[launch]
fn rocket() -> _ {
    id_gen::init(1);
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::on_ignite("mount_user", sample::init))
}
