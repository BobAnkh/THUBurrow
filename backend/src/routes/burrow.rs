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
    rocket.mount("/burrows", routes![burrow_create, burrow_discard])
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
                let valid_burrow_num = match get_burrow_list(state.valid_burrow.clone()).await {
                    Ok(valid_burrows) => valid_burrows.len() as i32,
                    Err(e) => {
                        error!(
                            "[CREATE BURROW] Failed to get valid burrow: {:?}",
                            e.to_string()
                        );
                        return (Status::InternalServerError, Err(String::new()));
                    }
                };
                let banned_burrow_num = match get_burrow_list(state.banned_burrow.clone()).await {
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

#[delete("/discard/<burrow_id>")]
pub async fn burrow_discard(
    db: Connection<PgDb>,
    burrow_id: i64,
    sso: sso::SsoAuth,
) -> (Status, Result<Json<i64>, String>) {
    let pg_con = db.into_inner();
    match pgdb::user_status::Entity::find_by_id(sso.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_ust) => match opt_ust {
            Some(state) => {
                let mut valid_burrows = match get_burrow_list(state.valid_burrow.clone()).await {
                    Ok(burrows_id) => burrows_id,
                    Err(e) => {
                        error!(
                            "[DEL BURROW] Failed to get valid burrows: {:?}",
                            e.to_string()
                        );
                        return (Status::InternalServerError, Err(String::new()));
                    }
                };
                let mut banned_burrows = match get_burrow_list(state.banned_burrow.clone()).await {
                    Ok(burrows_id) => burrows_id,
                    Err(e) => {
                        error!(
                            "[DEL BURROW] Failed to get valid burrows: {:?}",
                            e.to_string()
                        );
                        return (Status::InternalServerError, Err(String::new()));
                    }
                };
                let burrows_id = [valid_burrows.clone(), banned_burrows.clone()].concat();
                if burrows_id.contains(&burrow_id) {
                    // update valid_burrow / banned_burrow in user_status table
                    let mut ac_state: pgdb::user_status::ActiveModel = state.into();
                    // do some type-convert things, and fill in the row according to different situations
                    if valid_burrows.contains(&burrow_id) {
                        valid_burrows.remove(valid_burrows.binary_search(&burrow_id).unwrap());
                        let valid_burrows: Vec<String> =
                            valid_burrows.iter().map(|x| x.to_string()).collect();
                        let mut valid_burrows_str = valid_burrows.join(",") + ",";
                        if valid_burrows_str == "," {
                            valid_burrows_str = "".to_string();
                        }
                        ac_state.valid_burrow = Set(valid_burrows_str);
                    } else {
                        banned_burrows.remove(banned_burrows.binary_search(&burrow_id).unwrap());
                        let banned_burrows: Vec<String> =
                            banned_burrows.iter().map(|x| x.to_string()).collect();
                        let mut banned_burrows_str = banned_burrows.join(",") + ",";
                        if banned_burrows_str == "," {
                            banned_burrows_str = "".to_string();
                        }
                        ac_state.banned_burrow = Set(banned_burrows_str);
                    }
                    // update table user_status
                    match ac_state.update(&pg_con).await {
                        Ok(_) => {
                            info!("[DEL-BURROW] Table user_status updated.");
                            // update burrow_state in burrow table
                            match pgdb::burrow::Entity::find_by_id(burrow_id)
                                .one(&pg_con)
                                .await
                            {
                                Ok(opt_burrow) => match opt_burrow {
                                    Some(burrow) => {
                                        let mut ac_burrow: pgdb::burrow::ActiveModel =
                                            burrow.into();
                                        ac_burrow.burrow_state = Set(2);
                                        // update table burrow
                                        match ac_burrow.update(&pg_con).await {
                                            Ok(_) => {
                                                info!("[DEL-BURROW] Table burrow updated.");
                                                (Status::Ok, Ok(Json(burrow_id)))
                                            }
                                            Err(e) => {
                                                error!(
                                                    "[DEL-BURROW] Database Error: {:?}",
                                                    e.to_string()
                                                );
                                                return (
                                                    Status::InternalServerError,
                                                    Err(String::new()),
                                                );
                                            }
                                        }
                                    }
                                    None => {
                                        error!("[DEL-BURROW] Cannot find burrow by burrow_id.");
                                        return (Status::InternalServerError, Err(String::new()));
                                    }
                                },
                                Err(e) => {
                                    error!("[DEL-BURROW] Database Error: {:?}", e.to_string());
                                    return (Status::InternalServerError, Err(String::new()));
                                }
                            }
                        }
                        Err(e) => {
                            error!("[DEL-BURROW] Database Error: {:?}", e.to_string());
                            return (Status::InternalServerError, Err(String::new()));
                        }
                    }
                } else {
                    info!(
                        "[DEL-BURROW] Cannot delete burrow: Burrow doesn't belong to current user."
                    );
                    return (
                        Status::BadRequest,
                        Err(
                            "Burrow doesn't belong to current user or already discarded."
                                .to_string(),
                        ),
                    );
                }
            }
            None => {
                error!("[DEL-BURROW] Cannot find user_status by uid.");
                return (Status::InternalServerError, Err(String::new()));
            }
        },
        Err(e) => {
            error!("[DEL-BURROW] Database Error: {:?}", e.to_string());
            return (Status::InternalServerError, Err(String::new()));
        }
    }
}
