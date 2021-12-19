use chrono::{Duration, FixedOffset, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::{entity::*, DbErr};
use sea_orm::{query::*, DbBackend};

use crate::models::error::*;
use crate::models::pulsar::*;
use crate::models::{
    burrow::*,
    content::{Post, REPLY_PER_PAGE},
};
use crate::pgdb;
use crate::pool::{PgDb, PulsarSearchProducerMq};
use crate::utils::auth::Auth;
use crate::utils::burrow_valid::*;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/burrows",
        routes![
            create_burrow,
            discard_burrow,
            show_burrow,
            update_burrow,
            get_total_burrow_count
        ],
    )
}

#[get("/total")]
pub async fn get_total_burrow_count(
    _auth: Auth,
    db: Connection<PgDb>,
) -> (Status, Result<Json<BurrowTotalCount>, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    match LastBurrowSeq::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"SELECT "last_value" FROM "burrow_burrow_id_seq""#,
        vec![],
    ))
    .one(&pg_con)
    .await
    {
        Ok(r) => match r {
            Some(r) => (Status::Ok, Ok(Json(r.into()))),
            None => (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            ),
        },
        Err(e) => {
            log::error!("[TOTAL-BURROW] Database error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// Create Burrow
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `Json<BurrowInfo>`: Burrow information
/// - `Connection<PulsarSearchProducerMq>`: Pulsar connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `BurrowCreateResponse`: Response of create burrow
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::EmptyField`
///   - `ErrorCode::RateLimit`
///   - `ErrorCode::UserNotExist`
///   - `ErrorCode::UserForbidden`
///   - `ErrorCode::BurrowNumLimit`
///   - `ErrorCode::DatabaseErr`
///
#[post("/", data = "<burrow_info>", format = "json")]
pub async fn create_burrow(
    db: Connection<PgDb>,
    burrow_info: Json<BurrowInfo>,
    mut producer: Connection<PulsarSearchProducerMq>,
    auth: Auth,
) -> (
    Status,
    Result<Json<BurrowCreateResponse>, Json<ErrorResponse>>,
) {
    let pg_con = db.into_inner();
    // check if user has too many burrows, return corresponding error if so
    match pgdb::user_status::Entity::find_by_id(auth.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_state) => match opt_state {
            Some(state) => {
                if state.user_state != 0 {
                    return (
                        Status::Forbidden,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::UserForbidden,
                            "User not in a valid state",
                        ))),
                    );
                }
                let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
                // TODO: change it when final release
                if state
                    .update_time
                    .checked_add_signed(Duration::seconds(5))
                    .unwrap()
                    > now
                {
                    return (
                        Status::TooManyRequests,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::RateLimit,
                            "User can only create a new burrow every 24 hours",
                        ))),
                    );
                }
                let valid_burrows = get_burrow_list(&state.valid_burrow);
                let banned_burrows = get_burrow_list(&state.banned_burrow);
                if banned_burrows.len() + valid_burrows.len() < BURROW_LIMIT {
                    // get burrow info from request
                    let burrow = burrow_info.into_inner();
                    // check if Burrow Title is empty, return corresponding error if so
                    if burrow.title.is_empty() {
                        return (
                            Status::BadRequest,
                            Err(Json(ErrorResponse::build(
                                ErrorCode::EmptyField,
                                "Burrow title cannot be empty",
                            ))),
                        );
                    }
                    // fill the row of table 'burrow'
                    let burrows = pgdb::burrow::ActiveModel {
                        uid: Set(auth.id),
                        title: Set(burrow.title),
                        description: Set(burrow.description),
                        create_time: Set(now.to_owned()),
                        update_time: Set(now.to_owned()),
                        ..Default::default()
                    };
                    // insert the row in database
                    // <Fn, A, B> -> Result<A, B>
                    let mut ust: pgdb::user_status::ActiveModel = state.into();
                    match pg_con
                        .transaction::<_, BurrowCreateResponse, DbErr>(|txn| {
                            Box::pin(async move {
                                let res = burrows.insert(txn).await?;
                                let burrow_id = res.burrow_id.unwrap();
                                let pulsar_burrow = PulsarSearchBurrowData {
                                    burrow_id,
                                    title: res.title.unwrap(),
                                    description: res.description.unwrap(),
                                    update_time: now.to_owned(),
                                };
                                let uid = res.uid.unwrap();
                                ust.update_time = Set(now);
                                ust.valid_burrow = {
                                    let mut valid_burrows: Vec<i64> =
                                        get_burrow_list(&ust.valid_burrow.unwrap());
                                    valid_burrows.push(burrow_id);
                                    let valid_burrows_str = valid_burrows
                                        .iter()
                                        .map(|x| x.to_string())
                                        .collect::<Vec<String>>()
                                        .join(",");
                                    Set(valid_burrows_str)
                                };
                                ust.update(txn).await?;
                                info!(
                                    "[Create-Burrow] successfully create burrow {} for user {}",
                                    burrow_id, uid
                                );
                                // TODO: move them out of the transaction
                                let msg = PulsarSearchData::CreateBurrow(pulsar_burrow);
                                let _ = producer
                                    .send("persistent://public/default/search", msg)
                                    .await;
                                Ok(BurrowCreateResponse { burrow_id })
                            })
                        })
                        .await
                    {
                        Ok(resp) => (Status::Ok, Ok(Json(resp))),
                        Err(e) => {
                            error!("Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                } else {
                    info!("[CREATE-BURROW] Owned burrow amount reaches threshold.");
                    (
                        Status::Forbidden,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::BurrowNumLimit,
                            "Owned burrow amount is up to limit.",
                        ))),
                    )
                }
            }
            None => {
                info!("[CREATE BURROW] Cannot find user_status by uid.");
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                )
            }
        },
        Err(e) => {
            error!("[CREATE BURROW] Database Error: {:?}", e.to_string());
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// Discard Burrow
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `i64`: Burrow id
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::UserNotExist`
///   - `ErrorCode::UserForbidden`
///   - `ErrorCode::DatabaseErr`
///
#[delete("/<burrow_id>")]
pub async fn discard_burrow(
    db: Connection<PgDb>,
    burrow_id: i64,
    auth: Auth,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    match pgdb::user_status::Entity::find_by_id(auth.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_ust) => match opt_ust {
            Some(state) => {
                let mut valid_burrows: Vec<i64> = get_burrow_list(&state.valid_burrow);
                let mut banned_burrows: Vec<i64> = get_burrow_list(&state.banned_burrow);
                let mut ac_state: pgdb::user_status::ActiveModel = state.into();
                // update valid_burrow / banned_burrow in user_status table
                // do some type-convert things, and fill in the row according to different situations
                if valid_burrows.contains(&burrow_id) {
                    valid_burrows.remove(valid_burrows.binary_search(&burrow_id).unwrap());
                    let valid_burrows_str = valid_burrows
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(",");
                    ac_state.valid_burrow = Set(valid_burrows_str);
                    // update table user_status
                    match pg_con
                        .transaction::<_, (), DbErr>(|txn| {
                            Box::pin(async move {
                                ac_state.update(txn).await?;
                                let ac_burrow: pgdb::burrow::ActiveModel =
                                    pgdb::burrow::ActiveModel {
                                        burrow_id: Set(burrow_id),
                                        burrow_state: Set(2),
                                        ..Default::default()
                                    };
                                ac_burrow.update(txn).await?;
                                info!("[DISCARD-BURROW] Burrow {} discarded.", burrow_id);
                                Ok(())
                            })
                        })
                        .await
                    {
                        Ok(_) => (Status::Ok, Ok("Success".to_string())),
                        Err(e) => {
                            error!("Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                } else if banned_burrows.contains(&burrow_id) {
                    banned_burrows.remove(banned_burrows.binary_search(&burrow_id).unwrap());
                    let banned_burrows_str = banned_burrows
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(",");
                    ac_state.banned_burrow = Set(banned_burrows_str);
                    // update table user_status
                    match pg_con
                        .transaction::<_, (), DbErr>(|txn| {
                            Box::pin(async move {
                                ac_state.update(txn).await?;
                                let ac_burrow: pgdb::burrow::ActiveModel =
                                    pgdb::burrow::ActiveModel {
                                        burrow_id: Set(burrow_id),
                                        burrow_state: Set(3),
                                        ..Default::default()
                                    };
                                ac_burrow.update(txn).await?;
                                info!("[DISCARD-BURROW] Burrow {} discarded.", burrow_id);
                                Ok(())
                            })
                        })
                        .await
                    {
                        Ok(_) => (Status::Ok, Ok("Success".to_string())),
                        Err(e) => {
                            error!("Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                } else {
                    info!(
                        "[DEL-BURROW] Cannot delete burrow: Burrow doesn't belong to current user."
                    );
                    (
                        Status::Forbidden,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::UserForbidden,
                            "Burrow doesn't belong to current user or already be discarded",
                        ))),
                    )
                }
            }
            None => {
                info!("[DEL-BURROW] Cannot find user_status by uid.");
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                )
            }
        },
        Err(e) => {
            error!("[DEL-BURROW] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// Show a Specific Burrow with Up to Ten Posts
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `i64`: Burrow id
/// - `Option<usize>`: Page number for burrow
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `BurrowShowResponse`: Burrow detail information, including burrow information and up to 10 posts
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::BurrowNotExist`
///   - `ErrorCode::DatabaseErr`
///
#[get("/<burrow_id>?<page>")]
pub async fn show_burrow(
    db: Connection<PgDb>,
    burrow_id: i64,
    page: Option<usize>,
    _auth: Auth,
) -> (
    Status,
    Result<Json<BurrowShowResponse>, Json<ErrorResponse>>,
) {
    let pg_con = db.into_inner();
    let page = page.unwrap_or(0);
    match pgdb::burrow::Entity::find_by_id(burrow_id)
        .one(&pg_con)
        .await
    {
        Ok(opt_burrow) => match opt_burrow {
            Some(burrow) => {
                match pgdb::content_post::Entity::find()
                    .filter(pgdb::content_post::Column::BurrowId.eq(burrow_id))
                    .order_by_desc(pgdb::content_post::Column::PostId)
                    .paginate(&pg_con, REPLY_PER_PAGE)
                    .fetch_page(page)
                    .await
                {
                    Ok(posts) => (
                        Status::Ok,
                        Ok(Json(BurrowShowResponse {
                            title: burrow.title,
                            description: burrow.description,
                            posts: {
                                let posts_info: Vec<Post> =
                                    posts.iter().map(|post| post.into()).collect();
                                posts_info
                            },
                        })),
                    ),
                    Err(e) => {
                        error!("[SHOW-BURROW] Database Error: {:?}", e);
                        (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        )
                    }
                }
            }
            None => {
                info!("[SHOW-BURROW] Cannot find burrow {}", burrow_id);
                (
                    Status::NotFound,
                    Err(Json(ErrorResponse::build(ErrorCode::BurrowNotExist, ""))),
                )
            }
        },
        Err(e) => {
            error!("[SHOW-BURROW] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// Update Burrow
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `i64`: Burrow id
/// - `Json<BurrowInfo>`: Burrow information
/// - `Connection<PulsarSearchProducerMq>`: Pulsar connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::EmptyField`
///   - `ErrorCode::UserNotExist`
///   - `ErrorCode::UserForbidden`
///   - `ErrorCode::DatabaseErr`
///
#[patch("/<burrow_id>", data = "<burrow_info>", format = "json")]
pub async fn update_burrow(
    db: Connection<PgDb>,
    burrow_id: i64,
    burrow_info: Json<BurrowInfo>,
    mut producer: Connection<PulsarSearchProducerMq>,
    auth: Auth,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    let burrow = burrow_info.into_inner();
    if burrow.title.is_empty() {
        return (
            Status::BadRequest,
            Err(Json(ErrorResponse::build(
                ErrorCode::EmptyField,
                "Burrow title cannot be empty",
            ))),
        );
    }
    match pgdb::user_status::Entity::find_by_id(auth.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_ust) => match opt_ust {
            Some(state) => {
                if state.user_state != 0 {
                    (
                        Status::Forbidden,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::UserForbidden,
                            "User not in a valid state",
                        ))),
                    )
                } else if is_valid_burrow(&state.valid_burrow, &burrow_id) {
                    let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
                    let burrows = pgdb::burrow::ActiveModel {
                        burrow_id: Set(burrow_id),
                        title: Set(burrow.title.to_owned()),
                        description: Set(burrow.description.to_owned()),
                        update_time: Set(now.to_owned()),
                        ..Default::default()
                    };
                    let pulsar_burrow = PulsarSearchBurrowData {
                        burrow_id,
                        title: burrow.title,
                        description: burrow.description,
                        update_time: now,
                    };
                    match burrows.update(&pg_con).await {
                        Ok(_) => {
                            let msg = PulsarSearchData::UpdateBurrow(pulsar_burrow);
                            let _ = producer
                                .send("persistent://public/default/search", msg)
                                .await;
                            (Status::Ok, Ok("Success".to_string()))
                        }
                        Err(e) => {
                            error!("[UPDATE-BURROW] Database Error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                } else {
                    info!(
                        "[UPDATE-BURROW] Cannot update burrow: Burrow doesn't belong to current user."
                    );
                    (
                        Status::Forbidden,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::UserForbidden,
                            "Burrow doesn't belong to current user or already be discarded",
                        ))),
                    )
                }
            }
            None => {
                info!("[UPDATE-BURROW] Cannot find user_status by uid.");
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                )
            }
        },
        Err(e) => {
            error!("[UPDATE-BURROW] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}
