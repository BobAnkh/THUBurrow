use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::entity::*;

use chrono::{FixedOffset, Utc};

use crate::pgdb;
use crate::pool::PgDb;
use crate::req::burrow::*;
use crate::utils::get_valid_burrow::*;
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
    match pgdb::user_status::Entity::find_by_id(sso.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_state) => match opt_state {
            Some(state) => {
                let valid_burrow_num = match get_valid_burrow(&pg_con, state.uid).await {
                    Ok(valid_burrows) => valid_burrows.len() as i32,
                    Err(e) => {
                        error!(
                            "[CREATE BURROW] Failed to get valid burrow: {:?}",
                            e.to_string()
                        );
                        return (Status::InternalServerError, Err(String::new()));
                    }
                };
                let banned_burrow_num = match get_banned_burrow(&pg_con, state.uid).await {
                    Ok(banned_burrows) => banned_burrows.len() as i32,
                    Err(e) => {
                        error!(
                            "[CREATE BURROW] Failed to get valid burrow: {:?}",
                            e.to_string()
                        );
                        return (Status::InternalServerError, Err(String::new()));
                    }
                };
                let max_burrow_num: i32 = *BURROW_UP_THRE;
                if banned_burrow_num + valid_burrow_num < max_burrow_num {
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
                    match burrows.insert(&pg_con).await {
                        Ok(res) => {
                            let burrow_id = res.burrow_id.unwrap();
                            let uid = res.uid.unwrap();
                            // update modified time and valid burrows
                            let mut ust: pgdb::user_status::ActiveModel = state.into();
                            ust.update_time =
                                Set(Utc::now().with_timezone(&FixedOffset::east(8 * 3600)));
                            ust.valid_burrow =
                                Set(burrow_id.to_string() + "," + &ust.valid_burrow.unwrap());
                            match ust.update(&pg_con).await {
                                Ok(s) => {
                                    info!(
                                        "[Create-Burrow] Burrow create Succ, save burrow: {:?}",
                                        burrow_id
                                    );
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
                            error!("[Create-Burrow] Database Error: {:?}", e.to_string());
                            (Status::InternalServerError, Err("".to_string()))
                        }
                    }
                } else {
                    info!("[CREATE-BURROW] Owned burrow amount reaches threshold.");
                    return (
                        Status::BadRequest,
                        Err("Owned burrow amount reaches threshold.".to_string()),
                    );
                }
            }
            None => {
                error!("[CREATE BURROW] Cannot find user_status by uid.");
                return (Status::InternalServerError, Err(String::new()));
            }
        },
        Err(e) => {
            error!("[CREATE BURROW] Database Error: {:?}", e.to_string());
            return (Status::InternalServerError, Err(String::new()));
        }
    }
}
