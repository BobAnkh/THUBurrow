#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

use backend::cors;
use backend::pool::{PgDb, PulsarSearchProducerMq, RedisDb};
use backend::routes::{self, sample};
use backend::utils::id_gen;

#[launch]
fn rocket() -> _ {
    id_gen::init(1);
    let cors_handler = cors::init();
    rocket::build()
        .attach(PgDb::init())
        .attach(RedisDb::init())
        .attach(PulsarSearchProducerMq::init())
        .attach(cors_handler)
        .attach(AdHoc::on_ignite("mount_routes", routes::routes_init))
        .attach(AdHoc::on_ignite("mount_user", sample::init))
}
