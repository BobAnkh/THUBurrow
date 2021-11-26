use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use sea_orm::entity::*;

use chrono::{FixedOffset, Utc};

use crate::pgdb;
use crate::pool::PgDb;
use crate::req::burrow::*;
use crate::utils::sso;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/burrows", routes![burrow_create])
}

#[post("/create", data = "<burrow_info>", format = "json")]
pub async fn burrow_create(
    db: Connection<PgDb>,
    burrow_info: Json<BurrowInfo>,
    sso: sso::SsoAuth,
) -> (Status, Result<Json<BurrowCreateResponse>, String>) {
    let pg_con = db.into_inner();
    // check if user has too many burrows, return corresponding error if so
    match pgdb::user::Entity::find_by_id(sso.id).one(&pg_con) {
        Ok(user) => {

        },
        Err(e) => {
            error!("Database Error: {:?}", e.to_string());
            return (Stauts::InternalServerError, Json(Vec::new()));
        },
    }
    // get burrow info from request
    let burrow = burrow_info.into_inner();
    // check if Burrow Title is empty, return corresponding error if so
    if burrow.title == "".to_string() {
        return (
                Status::BadRequest,
                Err("Burrow title cannot be empty.".to_string()),
        );
    }
    // fill the row of table 'burrow'
    let burrows = pgdb::burrow::ActiveModel {
        uid: Set(sso.id),
        title: Set(burrow.title),
        description: Set(burrow.description),
        ..Default::default()
    };
    // insert the row in database
    let ins_result = burrows.insert(&pg_con).await;
    match ins_result {
        Ok(res) => {
            let burrow_id = res.burrow_id.unwrap();
            let uid = res.uid.unwrap();
            // update modified time and valid burrows
            let users_status = pgdb::user_status::Entity::find_by_id(uid)
                .one(&pg_con)
                .await;
            match users_status {
                Ok(ust) => {
                    let mut ust: pgdb::user_status::ActiveModel = ust.unwrap().into();
                    ust.modified_time = Set(Utc::now().with_timezone(&FixedOffset::east(8 * 3600)));
                    ust.valid_burrow = Set(ust.valid_burrow.unwrap() + "," + &burrow_id.to_string());
                    match ust.update(&pg_con).await {
                        Ok(s) => {
                            info!("[Create-Burrow] Burrow create Succ, save burrow: {:?}", burrow_id);
                            info!(
                                "[Create-Burrow] User Status Updated, uid: {}",
                                s.uid.unwrap()
                            );
                            (
                                Status::Ok,
                                Ok(Json(BurrowCreateResponse {
                                    burrow_id: burrow_id,
                                    title: res.title.unwrap(),
                                    uid: uid,
                                    description: res.description.unwrap(),
                                })),
                            )
                        }
                        Err(e) => {
                            error!("Database error: {:?}", e.to_string());
                            return (Status::InternalServerError, Err("".to_string()));
                        }
                    }
                }
                Err(e) => {
                    error!("Database error: {:?}", e.to_string());
                    return (Status::InternalServerError, Err("".to_string()));
                }
            }
        }
        _ => {
            error!("[Create-Burrow] Cannot insert burrow to postgres.");
            (Status::InternalServerError, Err("".to_string()))
        }
    }
}
