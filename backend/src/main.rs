#[macro_use]
extern crate rocket;

use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use rocket_db_pools::Database;

use backend::cors;
use backend::pool::{PgDb, RedisDb};
use backend::routes::{self, sample, user};
use backend::setup;
use backend::utils::id_gen;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &PgDb::fetch(&rocket).unwrap().connection;
    let _ = setup::create_post_table(conn).await;
    Ok(rocket)
}

#[launch]
fn rocket() -> _ {
    id_gen::init(1);
    let cors_handler = cors::init();
    rocket::build()
        .attach(PgDb::init())
        .attach(RedisDb::init())
        .attach(cors_handler)
        .attach(AdHoc::on_ignite("mount_routes", routes::routes_init))
        .attach(AdHoc::on_ignite("mount_user", sample::init))
        .attach(AdHoc::on_ignite("mount_user", user::init))
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
}
