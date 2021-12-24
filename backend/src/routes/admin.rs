//! Routes for admin

use chrono::{FixedOffset, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::{entity::*, ConnectionTrait, DbErr};

use crate::config::BACKEND_TEST_MODE;
use crate::db::{self, prelude::*};
use crate::models::pulsar::{
    PulsarSearchBurrowData, PulsarSearchData, PulsarSearchPostData, PulsarSearchReplyData,
};
use crate::models::{admin::*, error::*};
use crate::pool::{PgDb, PulsarMq};
use crate::utils::auth::Auth;
use crate::utils::burrow_valid::get_burrow_list;
use crate::utils::dedup::remove_duplicate;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    #[cfg(debug_assertions)]
    {
        let mut rocket = rocket.mount("/admin", routes![admin_operation]);
        if *BACKEND_TEST_MODE {
            rocket = rocket.mount("/admin", routes![admin_test]);
        }
        rocket
    }
    #[cfg(not(debug_assertions))]
    rocket.mount("/admin", routes![admin_operation])
}

/// Process admin operations
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `Json<AdminOperation>`: Admin operation
/// - `Connection<PulsarMq>`: Pulsar search producer connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: String "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::DatabaseErr`
///   - `ErrorCode::UserNotExist`
///   - `ErrorCode::UserForbidden`
///   - `ErrorCode::BurrowNotExist`
///   - `ErrorCode::PostNotExist`
///   - `ErrorCode::ReplyNotExist`
#[post("/", data = "<operation>", format = "json")]
pub async fn admin_operation(
    auth: Auth,
    db: Connection<PgDb>,
    operation: Json<AdminOperation>,
    mut producer: Connection<PulsarMq>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    let operation = operation.into_inner();
    match Admin::find_by_id(auth.id).one(&pg_con).await {
        Ok(admin) => match admin {
            Some(admin) => match operation {
                AdminOperation::BanUser { uid } => {
                    match UserStatus::find_by_id(uid).one(&pg_con).await {
                        Ok(user) => match user {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                            ),
                            Some(user) => {
                                if admin.role < user.permission {
                                    (
                                        Status::Forbidden,
                                        Err(Json(ErrorResponse::build(
                                            ErrorCode::UserForbidden,
                                            "Permission Denied.",
                                        ))),
                                    )
                                } else {
                                    let mut ust: db::user_status::ActiveModel = user.into();
                                    ust.user_state = Set(1);
                                    ust.permission = Set(admin.role);
                                    match ust.update(&pg_con).await {
                                        Ok(_) => (Status::Ok, Ok("Success".to_string())),
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::ReopenUser { uid } => {
                    match UserStatus::find_by_id(uid).one(&pg_con).await {
                        Ok(user) => match user {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                            ),
                            Some(user) => {
                                if admin.role < user.permission {
                                    (
                                        Status::Forbidden,
                                        Err(Json(ErrorResponse::build(
                                            ErrorCode::UserForbidden,
                                            "Permission Denied.",
                                        ))),
                                    )
                                } else {
                                    let mut ust: db::user_status::ActiveModel = user.into();
                                    ust.user_state = Set(0);
                                    ust.permission = Set(admin.role);
                                    match ust.update(&pg_con).await {
                                        Ok(_) => (Status::Ok, Ok("Success".to_string())),
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::BanBurrow { burrow_id } => {
                    match Burrow::find_by_id(burrow_id).one(&pg_con).await {
                        Ok(burrow) => {
                            match burrow {
                                None => (
                                    Status::BadRequest,
                                    Err(Json(ErrorResponse::build(ErrorCode::BurrowNotExist, ""))),
                                ),
                                Some(burrow) => {
                                    if admin.role < burrow.permission {
                                        (
                                            Status::Forbidden,
                                            Err(Json(ErrorResponse::build(
                                                ErrorCode::UserForbidden,
                                                "Permission Denied.",
                                            ))),
                                        )
                                    } else {
                                        let burrow_state = burrow.burrow_state;
                                        let burrow_id = burrow.burrow_id;
                                        let uid = burrow.uid;
                                        let mut bst: db::burrow::ActiveModel = burrow.into();
                                        bst.burrow_state = Set(burrow_state + 1 - burrow_state % 2);
                                        bst.permission = Set(admin.role);
                                        match pg_con
                                            .transaction::<_, (), DbErr>(|txn| {
                                                Box::pin(async move {
                                                    bst.update(txn).await?;
                                                    let ust = UserStatus::find_by_id(uid).one(txn).await?;
                                                    match ust {
                                                        None => {
                                                            log::error!("[ADMIN] User not found");
                                                            Err(DbErr::RecordNotFound("User not found".to_string()))
                                                        }
                                                        Some(ust) => {
                                                            let mut valid_burrows: Vec<i64> = get_burrow_list(&ust.valid_burrow);
                                                            let mut banned_burrows: Vec<i64> = get_burrow_list(&ust.banned_burrow);
                                                            let mut ust: db::user_status::ActiveModel = ust.into();
                                                            if valid_burrows.contains(&burrow_id) {
                                                                valid_burrows.remove(valid_burrows.binary_search(&burrow_id).unwrap());
                                                                banned_burrows.push(burrow_id);
                                                                banned_burrows = remove_duplicate(banned_burrows);
                                                                let valid_burrows_str = valid_burrows
                                                                    .iter()
                                                                    .map(|x| x.to_string())
                                                                    .collect::<Vec<String>>()
                                                                    .join(",");
                                                                let banned_burrows_str = banned_burrows
                                                                    .iter()
                                                                    .map(|x| x.to_string())
                                                                    .collect::<Vec<String>>()
                                                                    .join(",");
                                                                ust.valid_burrow = Set(valid_burrows_str);
                                                                ust.banned_burrow = Set(banned_burrows_str);
                                                                ust.update(txn).await?;
                                                            }
                                                            Ok(())
                                                        }
                                                    }
                                                })
                                            })
                                        .await
                                    {
                                        Ok(_) => {
                                            let msg = PulsarSearchData::DeleteBurrow(burrow_id);
                                            let _ = producer
                                                .send("persistent://public/default/search", msg)
                                                .await;
                                            (Status::Ok, Ok("Success".to_string()))
                                        }
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::ReopenBurrow { burrow_id } => {
                    match Burrow::find_by_id(burrow_id).one(&pg_con).await {
                        Ok(burrow) => {
                            match burrow {
                                None => (
                                    Status::BadRequest,
                                    Err(Json(ErrorResponse::build(ErrorCode::BurrowNotExist, ""))),
                                ),
                                Some(burrow) => {
                                    if admin.role < burrow.permission {
                                        (
                                            Status::Forbidden,
                                            Err(Json(ErrorResponse::build(
                                                ErrorCode::UserForbidden,
                                                "Permission Denied.",
                                            ))),
                                        )
                                    } else {
                                        let burrow_state = burrow.burrow_state;
                                        let uid = burrow.uid;
                                        let mut bst: db::burrow::ActiveModel = burrow.into();
                                        bst.burrow_state = Set(burrow_state - burrow_state % 2);
                                        bst.permission = Set(admin.role);
                                        match pg_con
                                            .transaction::<_, db::burrow::ActiveModel, DbErr>(|txn| {
                                                Box::pin(async move {
                                                    let bst = bst.update(txn).await?;
                                                    let ust = UserStatus::find_by_id(uid).one(txn).await?;
                                                    match ust {
                                                        None => {
                                                            log::error!("[ADMIN] User not found");
                                                            Err(DbErr::RecordNotFound("User not found".to_string()))
                                                        }
                                                        Some(ust) => {
                                                            let mut valid_burrows: Vec<i64> = get_burrow_list(&ust.valid_burrow);
                                                            let mut banned_burrows: Vec<i64> = get_burrow_list(&ust.banned_burrow);
                                                            let mut ust: db::user_status::ActiveModel = ust.into();
                                                            if banned_burrows.contains(&burrow_id) {
                                                                banned_burrows.remove(banned_burrows.binary_search(&burrow_id).unwrap());
                                                                valid_burrows.push(burrow_id);
                                                                valid_burrows = remove_duplicate(valid_burrows);
                                                                let valid_burrows_str = valid_burrows
                                                                    .iter()
                                                                    .map(|x| x.to_string())
                                                                    .collect::<Vec<String>>()
                                                                    .join(",");
                                                                let banned_burrows_str = banned_burrows
                                                                    .iter()
                                                                    .map(|x| x.to_string())
                                                                    .collect::<Vec<String>>()
                                                                    .join(",");
                                                                ust.valid_burrow = Set(valid_burrows_str);
                                                                ust.banned_burrow = Set(banned_burrows_str);
                                                                ust.update(txn).await?;
                                                            }
                                                            Ok(bst)
                                                        }
                                                    }
                                                })
                                            })
                                        .await
                                    {
                                        Ok(res) => {
                                            let pulsar_burrow = PulsarSearchBurrowData {
                                                burrow_id: res.burrow_id.unwrap(),
                                                title: res.title.unwrap(),
                                                description: res.description.unwrap(),
                                                update_time: res.update_time.unwrap(),
                                            };
                                            let msg = PulsarSearchData::CreateBurrow(pulsar_burrow);
                                            let _ = producer
                                                .send("persistent://public/default/search", msg)
                                                .await;
                                            (Status::Ok, Ok("Success".to_string()))
                                        }
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::BanPost { post_id } => {
                    match ContentPost::find_by_id(post_id).one(&pg_con).await {
                        Ok(post) => match post {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::PostNotExist, ""))),
                            ),
                            Some(post) => {
                                if admin.role < post.permission {
                                    (
                                        Status::Forbidden,
                                        Err(Json(ErrorResponse::build(
                                            ErrorCode::UserForbidden,
                                            "Permission Denied.",
                                        ))),
                                    )
                                } else {
                                    let mut pst: db::content_post::ActiveModel = post.into();
                                    pst.post_state = Set(1);
                                    pst.permission = Set(admin.role);
                                    match pst.update(&pg_con).await {
                                        Ok(_) => {
                                            let msg = PulsarSearchData::DeletePost(post_id);
                                            let _ = producer
                                                .send("persistent://public/default/search", msg)
                                                .await;
                                            (Status::Ok, Ok("Success".to_string()))
                                        }
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::ReopenPost { post_id } => {
                    match ContentPost::find_by_id(post_id).one(&pg_con).await {
                        Ok(post) => match post {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::PostNotExist, ""))),
                            ),
                            Some(post) => {
                                if admin.role < post.permission {
                                    (
                                        Status::Forbidden,
                                        Err(Json(ErrorResponse::build(
                                            ErrorCode::UserForbidden,
                                            "Permission Denied.",
                                        ))),
                                    )
                                } else {
                                    let mut pst: db::content_post::ActiveModel = post.into();
                                    pst.post_state = Set(0);
                                    pst.permission = Set(admin.role);
                                    match pst.update(&pg_con).await {
                                        Ok(content) => {
                                            let pulsar_post = PulsarSearchPostData {
                                                post_id,
                                                title: content.title.unwrap(),
                                                burrow_id: content.burrow_id.unwrap(),
                                                section: serde_json::from_str(
                                                    &content.section.unwrap(),
                                                )
                                                .unwrap(),
                                                tag: content
                                                    .tag
                                                    .unwrap()
                                                    .split(',')
                                                    .map(str::to_string)
                                                    .collect(),
                                                update_time: content.update_time.unwrap(),
                                            };
                                            let msg = PulsarSearchData::CreatePost(pulsar_post);
                                            let _ = producer
                                                .send("persistent://public/default/search", msg)
                                                .await;
                                            (Status::Ok, Ok("Success".to_string()))
                                        }
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::BanReply { post_id, reply_id } => {
                    match ContentReply::find_by_id((post_id, reply_id))
                        .one(&pg_con)
                        .await
                    {
                        Ok(reply) => match reply {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::ReplyNotExist, ""))),
                            ),
                            Some(reply) => {
                                if admin.role < reply.permission {
                                    (
                                        Status::Forbidden,
                                        Err(Json(ErrorResponse::build(
                                            ErrorCode::UserForbidden,
                                            "Permission Denied.",
                                        ))),
                                    )
                                } else {
                                    let mut rst: db::content_reply::ActiveModel = reply.into();
                                    rst.reply_state = Set(1);
                                    rst.permission = Set(admin.role);
                                    match rst.update(&pg_con).await {
                                        Ok(_) => {
                                            let msg =
                                                PulsarSearchData::DeleteReply(post_id, reply_id);
                                            let _ = producer
                                                .send("persistent://public/default/search", msg)
                                                .await;
                                            (Status::Ok, Ok("Success".to_string()))
                                        }
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::ReopenReply { post_id, reply_id } => {
                    match ContentReply::find_by_id((post_id, reply_id))
                        .one(&pg_con)
                        .await
                    {
                        Ok(reply) => match reply {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::ReplyNotExist, ""))),
                            ),
                            Some(reply) => {
                                if admin.role < reply.permission {
                                    (
                                        Status::Forbidden,
                                        Err(Json(ErrorResponse::build(
                                            ErrorCode::UserForbidden,
                                            "Permission Denied.",
                                        ))),
                                    )
                                } else {
                                    let mut rst: db::content_reply::ActiveModel = reply.into();
                                    rst.reply_state = Set(0);
                                    rst.permission = Set(admin.role);
                                    match rst.update(&pg_con).await {
                                        Ok(content) => {
                                            let pulsar_reply = PulsarSearchReplyData {
                                                post_id,
                                                reply_id,
                                                burrow_id: content.burrow_id.unwrap(),
                                                content: content.content.unwrap(),
                                                update_time: content.update_time.unwrap(),
                                            };
                                            let msg = PulsarSearchData::CreateReply(pulsar_reply);
                                            let _ = producer
                                                .send("persistent://public/default/search", msg)
                                                .await;
                                            (Status::Ok, Ok("Success".to_string()))
                                        }
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::CreateAdmin { uid } => {
                    if admin.role > 0 {
                        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
                        let new_admin = db::admin::ActiveModel {
                            uid: Set(uid),
                            create_time: Set(now),
                            ..Default::default()
                        };
                        match new_admin.insert(&pg_con).await {
                            Ok(_) => (Status::Ok, Ok("Success".to_string())),
                            Err(e) => {
                                log::error!("[ADMIN] Database Error: {:?}", e);
                                (
                                    Status::InternalServerError,
                                    Err(Json(ErrorResponse::default())),
                                )
                            }
                        }
                    } else {
                        (
                            Status::Forbidden,
                            Err(Json(ErrorResponse::build(
                                ErrorCode::UserForbidden,
                                "Permission Denied.",
                            ))),
                        )
                    }
                }
                AdminOperation::DeleteAdmin { uid } => {
                    match Admin::find_by_id(uid).one(&pg_con).await {
                        Ok(admin_opt) => match admin_opt {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                            ),
                            Some(admin_opt) => {
                                if admin.role > admin_opt.role {
                                    let admin_opt: db::admin::ActiveModel = admin_opt.into();
                                    match admin_opt.delete(&pg_con).await {
                                        Ok(_) => (Status::Ok, Ok("Success".to_string())),
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                } else {
                                    (
                                        Status::Forbidden,
                                        Err(Json(ErrorResponse::build(
                                            ErrorCode::UserForbidden,
                                            "Permission Denied.",
                                        ))),
                                    )
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[ADMIN] Database Error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::SetAdminRole { uid, role } => {
                    match Admin::find_by_id(uid).one(&pg_con).await {
                        Ok(admin_opt) => match admin_opt {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                            ),
                            Some(admin_opt) => {
                                if admin.role > admin_opt.role && admin.role > role {
                                    let mut admin_opt: db::admin::ActiveModel = admin_opt.into();
                                    admin_opt.role = Set(role);
                                    match admin_opt.update(&pg_con).await {
                                        Ok(_) => (Status::Ok, Ok("Success".to_string())),
                                        Err(e) => {
                                            log::error!("[ADMIN] Database Error: {:?}", e);
                                            (
                                                Status::InternalServerError,
                                                Err(Json(ErrorResponse::default())),
                                            )
                                        }
                                    }
                                } else {
                                    (
                                        Status::Forbidden,
                                        Err(Json(ErrorResponse::build(
                                            ErrorCode::UserForbidden,
                                            "Permission Denied.",
                                        ))),
                                    )
                                }
                            }
                        },
                        Err(e) => {
                            log::error!("[ADMIN] Database Error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
                AdminOperation::GetUserId { burrow_id } => {
                    match Burrow::find_by_id(burrow_id).one(&pg_con).await {
                        Ok(burrow) => match burrow {
                            None => (
                                Status::BadRequest,
                                Err(Json(ErrorResponse::build(ErrorCode::BurrowNotExist, ""))),
                            ),
                            Some(burrow) => (Status::Ok, Ok(burrow.uid.to_string())),
                        },
                        Err(e) => {
                            log::error!("[ADMIN]: Database error: {:?}", e);
                            (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            )
                        }
                    }
                }
            },
            None => (
                Status::Forbidden,
                Err(Json(ErrorResponse::build(
                    ErrorCode::UserForbidden,
                    "Permission denied.",
                ))),
            ),
        },
        Err(e) => {
            log::error!("[ADMIN] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// Set Admin account when in test
///
/// ## Parameters
///
/// - `auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `role`: Admin role
///
/// ## Returns
///
/// - `Status`: Response status
/// - `String`: String "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::DatabaseErr`
#[cfg(debug_assertions)]
#[get("/test?<role>")]
pub async fn admin_test(
    auth: Auth,
    db: Connection<PgDb>,
    role: i32,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
    let admin = db::admin::ActiveModel {
        uid: Set(auth.id),
        role: Set(role),
        create_time: Set(now),
    };
    match admin.insert(&pg_con).await {
        Ok(_) => (Status::Ok, Ok("Success".to_string())),
        Err(e) => {
            log::error!("[ADMIN] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}
