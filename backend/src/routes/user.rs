use chrono::{FixedOffset, Utc};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use idgenerator::IdHelper;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::sea_query::{Expr, Query};
use sea_orm::{entity::*, query::*};
use sea_orm::{DbErr, QueryFilter};
use std::collections::HashMap;
use std::iter;

use crate::models::{
    burrow::{BurrowMetadata, BURROW_PER_PAGE},
    content::{Post, POST_PER_PAGE},
    error::*,
    pulsar::*,
    user::*,
};
use crate::pgdb;
use crate::pgdb::prelude::*;
use crate::pool::{PgDb, PulsarSearchProducerMq, RedisDb};
use crate::utils::auth::{Auth, CookieOptions};
use crate::utils::burrow_valid::*;
use crate::utils::email;
// use crate::utils::email_send;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/users",
        routes![
            user_log_in,
            user_sign_up,
            get_follow,
            get_collection,
            get_burrow,
            user_relation,
            get_user_valid_burrow,
            user_email_activate,
        ],
    )
}

async fn gen_salt() -> String {
    let salt: String = iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(8)
        .collect();
    salt
}

#[post("/relation", data = "<relation_info>", format = "json")]
pub async fn user_relation(
    auth: Auth,
    mut producer: Connection<PulsarSearchProducerMq>,
    relation_info: Json<RelationData>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let relation = relation_info.into_inner();
    let msg = relation.to_pulsar(auth.id);
    match producer
        .send("persistent://public/default/relation", msg)
        .await
    {
        Ok(_) => log::info!("[RELATION] send data to pulsar successfully!"),
        Err(e) => {
            log::error!("[RELATION] PulsarErr: {:?}", e);
            return (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            );
        }
    }
    (Status::Ok, Ok("Success".to_string()))
}

// #[get("/clear")]
// pub async fn clear_redis(kvdb: Connection<RedisDb>) {
//     let email = "gsr18@mails.tsinghua.edu.cn".to_string();
//     let mut kvdb_con = kvdb.into_inner();
//     let _: Result<String, redis::RedisError> = redis::cmd("SET")
//         .arg(&email)
//         .arg("0".to_string() + ":" + "666666")
//         .query_async(kvdb_con.as_mut())
//         .await;
// }

#[post("/email", data = "<email_info>", format = "json")]
pub async fn user_email_activate(
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    email_info: Json<UserEmail>,
    mut producer: Connection<PulsarSearchProducerMq>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    let mut kvdb_con = kvdb.into_inner();
    let email = email_info.into_inner().email;
    if !email::check_email_syntax(&email) {
        return (
            Status::BadRequest,
            Err(Json(ErrorResponse::build(
                ErrorCode::EmailInvalid,
                "Invalid Email address",
            ))),
        );
    }
    // check if email address is duplicated, add corresponding error if so
    match User::find()
        .filter(pgdb::user::Column::Email.eq(email.clone()))
        .one(&pg_con)
        .await
    {
        Ok(res) => {
            if res.is_some() {
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(
                        ErrorCode::EmailDuplicate,
                        "This Email address is already in use",
                    ))),
                )
            } else {
                let get_redis_result: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
                    .arg(&email)
                    .query_async(kvdb_con.as_mut())
                    .await;
                let mut op_times;
                match get_redis_result {
                    Ok(opt_res) => match opt_res {
                        Some(res) => {
                            let values: Vec<&str> = res.split(':').collect();
                            op_times = values[0].parse::<usize>().unwrap();
                        }
                        None => {
                            op_times = 0;
                        }
                    },
                    Err(e) => {
                        log::error!("[EMAIL-AC] Database Error: {:?}", e);
                        return (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        );
                    }
                };
                op_times += 1;
                log::info!("[EMAIL-AC] op_times: {}", op_times);
                if op_times > SEND_EMAIL_LIMIT {
                    return (
                        Status::TooManyRequests,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::RateLimit,
                            "Request Send-Email too many times",
                        ))),
                    );
                }
                // match email_send::post(email.clone(), 666666).await {
                //     Ok(res) => {
                //         println!("{}", res);
                //     },
                //     Err(e) => {
                //         println!("{}", e);
                //     },
                // };
                let msg = PulsarSendEmail { email };
                match producer
                    .send("persistent://public/default/email", msg)
                    .await
                {
                    Ok(_) => (Status::Ok, Ok("Success".to_string())),
                    Err(e) => {
                        log::error!("[EMAIL-AC] Database error: {:?}", e);
                        (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        )
                    }
                }
            }
        }
        Err(e) => {
            log::error!("[EMAIL-AC] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

#[post("/sign-up", data = "<user_info>", format = "json")]
pub async fn user_sign_up(
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    user_info: Json<UserInfo<'_>>,
) -> (Status, Result<Json<UserResponse>, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    let mut kvdb_con = kvdb.into_inner();
    // get user info from request
    let user = user_info.into_inner();
    // check if email address is valid, add corresponding error if so
    if !email::check_email_syntax(user.email) {
        (
            Status::BadRequest,
            Err(Json(ErrorResponse::build(
                ErrorCode::EmailInvalid,
                "Invalid Email address.",
            ))),
        )
    } else if user.username.is_empty() {
        (
            Status::BadRequest,
            Err(Json(ErrorResponse::build(
                ErrorCode::EmptyField,
                "Empty username.",
            ))),
        )
    } else {
        // check if email address is duplicated, add corresponding error if so
        match User::find()
            .filter(pgdb::user::Column::Email.eq(user.email))
            .one(&pg_con)
            .await
        {
            Ok(res) => {
                if res.is_some() {
                    return (
                        Status::BadRequest,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::EmailDuplicate,
                            "Duplicate Email address.",
                        ))),
                    );
                }
            }
            Err(e) => {
                log::error!("[SIGN-UP] Database Error: {:?}", e);
                return (
                    Status::InternalServerError,
                    Err(Json(ErrorResponse::default())),
                );
            }
        }
        // check if username is duplicated, add corresponding error if so
        match User::find()
            .filter(pgdb::user::Column::Username.eq(user.username))
            .one(&pg_con)
            .await
        {
            Ok(res) => {
                if res.is_some() {
                    return (
                        Status::BadRequest,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::UsernameDuplicate,
                            "Duplicate username.",
                        ))),
                    );
                }
            }
            Err(e) => {
                log::error!("[SIGN-UP] Database Error: {:?}", e);
                return (
                    Status::InternalServerError,
                    Err(Json(ErrorResponse::default())),
                );
            }
        }
        // check if verification code is correct, return corresponding error if so
        let get_redis_result: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
            .arg(&user.email)
            .query_async(kvdb_con.as_mut())
            .await;
        match get_redis_result {
            Ok(res) => match res {
                Some(s) => {
                    let values: Vec<&str> = s.split(':').collect();
                    let code = values[1];
                    if !user.verification_code.eq(code) {
                        return (
                            Status::BadRequest,
                            Err(Json(ErrorResponse::build(
                                ErrorCode::CredentialInvalid,
                                "Invalid verification code: Wrong verification code.",
                            ))),
                        );
                    }
                }
                None => {
                    return (
                        Status::BadRequest,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::CredentialInvalid,
                            "Invalid verification code: Cannot find email -> code in redis.",
                        ))),
                    )
                }
            },
            Err(e) => {
                log::error!("[SIGN-UP] Database Error: {:?}", e);
                return (
                    Status::InternalServerError,
                    Err(Json(ErrorResponse::default())),
                );
            }
        };

        // generate salt
        let salt = gen_salt().await;
        // encrypt password
        let mut hash_sha3 = Sha3::sha3_256();
        hash_sha3.input_str(&(String::from(&salt) + user.password));
        let password = hash_sha3.result_str();
        // generate uid
        let uid: i64 = IdHelper::next_id();
        // fill the row of table 'user' and 'user_status'
        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let users = pgdb::user::ActiveModel {
            uid: Set(uid),
            username: Set(user.username.to_string()),
            password: Set(password),
            email: Set(user.email.to_string()),
            create_time: Set(now),
            salt: Set(salt),
        };

        let burrows = pgdb::burrow::ActiveModel {
            uid: Set(uid),
            title: Set("Default".to_owned()),
            description: Set("".to_owned()),
            ..Default::default()
        };
        // insert rows in database
        // <Fn, A, B> -> Result<A, B>
        match pg_con
            .transaction::<_, i64, DbErr>(|txn| {
                Box::pin(async move {
                    users.insert(txn).await?;
                    let res = burrows.insert(txn).await?;
                    let burrow_id = res.burrow_id.unwrap();
                    let valid_burrows_str = burrow_id.to_string();
                    let users_status = pgdb::user_status::ActiveModel {
                        uid: Set(uid),
                        update_time: Set(now),
                        valid_burrow: Set(valid_burrows_str),
                        ..Default::default()
                    };
                    users_status.insert(txn).await?;
                    Ok(burrow_id)
                })
            })
            .await
        {
            Ok(default_burrow) => (Status::Ok, Ok(Json(UserResponse { default_burrow }))),
            Err(e) => {
                error!("[SIGN-UP] Database error: {:?}", e);
                (
                    Status::InternalServerError,
                    Err(Json(ErrorResponse::default())),
                )
            }
        }
    }
}

#[post("/login", data = "<user_info>", format = "json")]
pub async fn user_log_in(
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    cookies: &CookieJar<'_>,
    user_info: Json<UserLoginInfo<'_>>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let mut con = kvdb.into_inner();
    // get user info from request
    let user = user_info.into_inner();
    // check if username is existed, add corresponding error if so
    match User::find()
        .filter(pgdb::user::Column::Username.eq(user.username))
        .one(&db.into_inner())
        .await
    {
        Ok(s) => match s {
            Some(matched_user) => {
                info!("[LOGIN] username exists, continue...");
                let salt = matched_user.salt;
                if salt.is_empty() {
                    error!("[LOGIN] cannot find user's salt.");
                    return (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    );
                }
                // encrypt input password same as sign-up
                let mut hash_sha3 = Sha3::sha3_256();
                hash_sha3.input_str(&(salt + user.password));
                let password = hash_sha3.result_str();
                // check if password is wrong, add corresponding error if so
                if matched_user.password.eq(&password) {
                    info!("[LOGIN] password correct, continue...");
                    // generate token and refresh token
                    let token: String = iter::repeat(())
                        .map(|()| thread_rng().sample(Alphanumeric))
                        .map(char::from)
                        .take(32)
                        .collect();
                    let mut hash_sha3 = Sha3::sha3_384();
                    hash_sha3.input_str(&token);
                    let refresh_token = hash_sha3.result_str();
                    // set token -> id
                    let uid_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                        .arg(&token)
                        .arg(4 * 3600i32)
                        .arg(matched_user.uid)
                        .query_async(con.as_mut())
                        .await;
                    match uid_result {
                        Ok(s) => info!("[LOGIN] setex token->id: {:?} -> {}", &token, s),
                        Err(e) => {
                            error!(
                                "[LOGIN] failed to set token -> id when login. RedisError: {:?}",
                                e
                            );
                            return (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            );
                        }
                    };
                    // set refresh_token -> id
                    let uid_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                        .arg(&refresh_token)
                        .arg(15 * 24 * 3600i32)
                        .arg(matched_user.uid)
                        .query_async(con.as_mut())
                        .await;
                    match uid_result {
                        Ok(s) => info!(
                            "[LOGIN] setex refresh_token->id: {:?} -> {}",
                            &refresh_token, s
                        ),
                        Err(e) => {
                            error!("[LOGIN] failed to set refresh_token -> id when login. RedisError: {:?}", e);
                            return (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            );
                        }
                    };
                    // get old token and set new token by getset id -> token
                    // TODO: add time limit to the key?
                    let old_token_get: Result<Option<String>, redis::RedisError> =
                        redis::cmd("GETSET")
                            .arg(matched_user.uid)
                            .arg(&token)
                            .query_async(con.as_mut())
                            .await;
                    match old_token_get {
                        Ok(res) => match res {
                            // if old token -> id exists
                            Some(old_token) => {
                                info!("[LOGIN] find old token:{:?}, continue...", old_token);
                                // clear old token -> id
                                let delete_result: Result<i64, redis::RedisError> =
                                    redis::cmd("DEL")
                                        .arg(&old_token)
                                        .query_async(con.as_mut())
                                        .await;
                                match delete_result {
                                    Ok(1) => info!("[LOGIN] delete token->id"),
                                    Ok(0) => info!("[LOGIN] no token->id found"),
                                    Ok(_) => {
                                        error!("[LOGIN] failed to delete refresh_token -> id when login.");
                                        return (
                                            Status::InternalServerError,
                                            Err(Json(ErrorResponse::default())),
                                        );
                                    }
                                    Err(e) => {
                                        error!("[LOGIN] failed to delete token -> id when login. RedisError: {:?}", e);
                                        return (
                                            Status::InternalServerError,
                                            Err(Json(ErrorResponse::default())),
                                        );
                                    }
                                };
                                // find old refresh_token by hashing old token
                                let mut hash_sha3 = Sha3::sha3_384();
                                hash_sha3.input_str(&old_token);
                                let old_refresh_token = hash_sha3.result_str();
                                // clear old refresh_token -> id
                                let delete_result: Result<i64, redis::RedisError> =
                                    redis::cmd("DEL")
                                        .arg(&old_refresh_token)
                                        .query_async(con.as_mut())
                                        .await;
                                match delete_result {
                                    Ok(1) => info!("[LOGIN] delete ref_token->id"),
                                    Ok(0) => info!("[LOGIN] no ref_token->id found"),
                                    Ok(_) => {
                                        error!("[LOGIN] failed to delete refresh_token -> id when login.");
                                        return (
                                            Status::InternalServerError,
                                            Err(Json(ErrorResponse::default())),
                                        );
                                    }
                                    Err(e) => {
                                        error!("[LOGIN] failed to delete refresh_token -> id when login. RedisError: {:?}", e);
                                        return (
                                            Status::InternalServerError,
                                            Err(Json(ErrorResponse::default())),
                                        );
                                    }
                                };
                                info!("[LOGIN] set id->token: {} -> {:?}", matched_user.uid, token);
                            }
                            None => {
                                info!("[LOGIN] no id->token found");
                                info!("[LOGIN] set id->token: {} -> {:?}", matched_user.uid, token);
                            }
                        },
                        Err(e) => {
                            error!(
                                "[LOGIN] failed to set id -> token when login. RedisError: {:?}",
                                e
                            );
                            return (
                                Status::InternalServerError,
                                Err(Json(ErrorResponse::default())),
                            );
                        }
                    };
                    // build cookie
                    let cookie = Cookie::build("token", token).cookie_options().finish();
                    // set cookie
                    cookies.add_private(cookie);
                    info!("[LOGIN] User login complete.");
                    (Status::Ok, Ok("Success".to_string()))
                } else {
                    info!("[LOGIN] wrong password.");
                    (
                        Status::BadRequest,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::CredentialInvalid,
                            "Wrong username or password.",
                        ))),
                    )
                }
            }
            None => {
                info!("[LOGIN] username does not exists.");
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(
                        ErrorCode::CredentialInvalid,
                        "Wrong username or password.",
                    ))),
                )
            }
        },
        Err(e) => {
            error!("[LOGIN] Database error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

#[get("/burrows")]
pub async fn get_burrow(
    db: Connection<PgDb>,
    auth: Auth,
) -> (
    Status,
    Result<Json<Vec<BurrowMetadata>>, Json<ErrorResponse>>,
) {
    // Ok(burrows) => {
    //     // let mut posts_num = Vec::new();
    //     let r: Vec<i64> = future::try_join_all(burrows.iter().map(move |burrow| {
    //         let inner_conn = pg_con.clone();
    //         GetBurrow::get_post(burrow, inner_conn)
    //     }))
    //     .await
    //     .unwrap();
    let pg_con = db.into_inner();
    match pgdb::user_status::Entity::find_by_id(auth.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_state) => match opt_state {
            Some(state) => {
                let valid_burrows = get_burrow_list(&state.valid_burrow);
                let banned_burrows = get_burrow_list(&state.banned_burrow);
                let burrow_ids = [valid_burrows, banned_burrows].concat();
                match Burrow::find()
                    .filter(Condition::any().add(pgdb::burrow::Column::BurrowId.is_in(burrow_ids)))
                    .order_by_desc(pgdb::burrow::Column::BurrowId)
                    .all(&pg_con)
                    .await
                {
                    Ok(burrows) => (
                        Status::Ok,
                        Ok(Json(burrows.iter().map(|burrow| burrow.into()).collect())),
                    ),
                    Err(e) => {
                        error!("[GET_BURROW] failed to get burrow list: {:?}", e);
                        (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        )
                    }
                }
            }
            None => {
                info!("[GET-BURROW] Cannot find user_status by uid.");
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                )
            }
        },
        Err(e) => {
            error!("[GET-BURROW] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

#[get("/collection?<page>")]
pub async fn get_collection(
    db: Connection<PgDb>,
    auth: Auth,
    page: Option<u64>,
) -> (Status, Result<Json<Vec<Post>>, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    let page = page.unwrap_or(0);
    match ContentPost::find()
        .filter(
            Condition::any().add(
                pgdb::content_post::Column::PostId.in_subquery(
                    Query::select()
                        .column(pgdb::user_collection::Column::PostId)
                        .from(UserCollection)
                        .and_where(Expr::col(pgdb::user_collection::Column::Uid).eq(auth.id))
                        .order_by_columns(vec![(
                            pgdb::user_collection::Column::PostId,
                            Order::Desc,
                        )])
                        .limit(POST_PER_PAGE as u64)
                        .offset(page * POST_PER_PAGE as u64)
                        .to_owned(),
                ),
            ),
        )
        .order_by_desc(pgdb::content_post::Column::PostId)
        .all(&pg_con)
        .await
    {
        Ok(posts) => (
            Status::Ok,
            Ok(Json(posts.iter().map(|post| post.into()).collect())),
        ),
        Err(e) => {
            error!("[GET-FAV] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

#[get("/follow?<page>")]
pub async fn get_follow(
    db: Connection<PgDb>,
    auth: Auth,
    page: Option<usize>,
) -> (
    Status,
    Result<Json<Vec<UserGetFollowResponse>>, Json<ErrorResponse>>,
) {
    let pg_con = db.into_inner();
    let page = page.unwrap_or(0);
    match pgdb::user_follow::Entity::find()
        .filter(pgdb::user_follow::Column::Uid.eq(auth.id))
        .order_by_desc(pgdb::user_follow::Column::BurrowId)
        .paginate(&pg_con, BURROW_PER_PAGE)
        .fetch_page(page)
        .await
    {
        Ok(results) => {
            let burrow_ids = results.iter().map(|r| r.burrow_id).collect::<Vec<i64>>();
            match pgdb::burrow::Entity::find()
                .filter(pgdb::burrow::Column::BurrowId.is_in(burrow_ids))
                .order_by_desc(pgdb::burrow::Column::BurrowId)
                .all(&pg_con)
                .await
            {
                Ok(burrows) => {
                    if burrows.len() == results.len() {
                        (
                            Status::Ok,
                            Ok(Json(
                                burrows
                                    .iter()
                                    .map(|burrow| {
                                        let meta: BurrowMetadata = burrow.into();
                                        meta
                                    })
                                    .zip(results.iter().map(|r| r.is_update))
                                    .map(|(burrow, is_update)| UserGetFollowResponse {
                                        burrow,
                                        is_update,
                                    })
                                    .collect(),
                            )),
                        )
                    } else {
                        let hm: HashMap<i64, bool> =
                            results.iter().map(|r| (r.burrow_id, r.is_update)).collect();
                        (
                            Status::Ok,
                            Ok(Json(
                                burrows
                                    .iter()
                                    .map(|meta| {
                                        let burrow: BurrowMetadata = meta.into();
                                        let is_update =
                                            *hm.get(&burrow.burrow_id).unwrap_or(&false);
                                        UserGetFollowResponse { burrow, is_update }
                                    })
                                    .collect(),
                            )),
                        )
                    }
                }
                Err(e) => {
                    error!("[GET-FOLLOW] Database Error: {:?}", e);
                    (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    )
                }
            }
        }
        Err(e) => {
            error!("[GET-FOLLOW] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

#[get("/valid-burrows")]
pub async fn get_user_valid_burrow(
    auth: Auth,
    db: Connection<PgDb>,
) -> (Status, Result<Json<Vec<i64>>, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    match pgdb::user_status::Entity::find_by_id(auth.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_state) => match opt_state {
            Some(state) => (Status::Ok, Ok(Json(get_burrow_list(&state.valid_burrow)))),
            None => {
                info!("[GET-VALID-BURROW] Cannot find user_status by uid.");
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                )
            }
        },
        Err(e) => {
            error!("[GET-VALID-BURROW] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}
