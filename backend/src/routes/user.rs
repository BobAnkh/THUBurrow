//! Routes for user
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
use sea_orm::{entity::*, query::*, DbErr, QueryFilter};
use std::collections::HashMap;

use crate::config::burrow::BURROW_PER_PAGE;
use crate::config::content::POST_PER_PAGE;
use crate::config::user::SEND_EMAIL_LIMIT;
use crate::db::{self, prelude::*};
use crate::models::{burrow::BurrowMetadata, content::Post, error::*, pulsar::*, user::*};
use crate::pool::{PgDb, PulsarMq, RedisDb};
use crate::utils::auth::{delete_token, set_token, Auth, CookieOptions};
use crate::utils::burrow_valid::*;
use crate::utils::email;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/users",
        routes![
            user_log_in,
            user_sign_up,
            user_logout,
            get_follow,
            get_collection,
            get_burrow,
            user_relation,
            get_user_valid_burrow,
            user_email_activate,
            user_reset,
            user_reset_email,
            user_change_password,
        ],
    )
}

async fn gen_salt() -> String {
    let salt: String = std::iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(8)
        .collect();
    salt
}

/// User Relation
///
/// User likes/dislikes a post, adds a post to user collection, removes a post from user collection, follows/unfollows a burrow.
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PulsarMq>`: Pulsar connection
/// - `Json<RelationData>`: Json of relation between user and certain post/burrow
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
#[post("/relation", data = "<relation_info>", format = "json")]
pub async fn user_relation(
    auth: Auth,
    mut producer: Connection<PulsarMq>,
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

/// User Email Activate
///
/// Send verification email for user sign-up, allow 3 requests each 4 hours.
///
/// ## Parameters
///
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<RedisDb>`: Redis connection
/// - `Json<UserEmail>`: Json of user email
/// - `Connection<PulsarMq>`: Pulsar connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: String "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::EmailInvalid`
///   - `ErrorCode::EmailDuplicate`
///   - `ErrorCode::RateLimit`
///   - `ErrorCode::DatabaseErr`
#[post("/email", data = "<email_info>", format = "json")]
pub async fn user_email_activate(
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    email_info: Json<UserEmail>,
    mut producer: Connection<PulsarMq>,
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
        .filter(db::user::Column::Email.eq(email.clone()))
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
                let op_times = 1 + match get_redis_result {
                    Ok(opt_res) => match opt_res {
                        Some(res) => {
                            let values: Vec<&str> = res.split(':').collect();
                            values[0].parse::<usize>().unwrap()
                        }
                        None => 0,
                    },
                    Err(e) => {
                        log::error!("[EMAIL-AC] Database Error: {:?}", e);
                        return (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        );
                    }
                };
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
                let msg = PulsarSendEmail::Sign { email };
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

/// User Reset Email
///
/// Send verification email for password-reset, allow 3 requests each 4 hours.
///
/// ## Parameters
///
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<RedisDb>`: Redis connection
/// - `Json<UserEmail>`: Json of user email
/// - `Connection<PulsarMq>`: Pulsar connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: String "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::EmailInvalid`
///   - `ErrorCode::EmailDuplicate`
///   - `ErrorCode::RateLimit`
///   - `ErrorCode::DatabaseErr`
#[post("/reset/email", data = "<email_info>", format = "json")]
pub async fn user_reset_email(
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    email_info: Json<UserEmail>,
    mut producer: Connection<PulsarMq>,
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
    // check if email address is exist, add corresponding error if so
    match User::find()
        .filter(db::user::Column::Email.eq(email.clone()))
        .one(&pg_con)
        .await
    {
        Ok(res) => {
            if res.is_some() {
                let get_redis_result: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
                    .arg(&email)
                    .query_async(kvdb_con.as_mut())
                    .await;
                let op_times = 1 + match get_redis_result {
                    Ok(opt_res) => match opt_res {
                        Some(res) => {
                            let values: Vec<&str> = res.split(':').collect();
                            values[0].parse::<usize>().unwrap()
                        }
                        None => 0,
                    },
                    Err(e) => {
                        log::error!("[EMAIL-RESET] Database Error: {:?}", e);
                        return (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        );
                    }
                };
                log::info!("[EMAIL-RESET] op_times: {}", op_times);
                if op_times > SEND_EMAIL_LIMIT {
                    return (
                        Status::TooManyRequests,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::RateLimit,
                            "Request Send-Email too many times",
                        ))),
                    );
                }
                let msg = PulsarSendEmail::Reset { email };
                match producer
                    .send("persistent://public/default/email", msg)
                    .await
                {
                    Ok(_) => (Status::Ok, Ok("Success".to_string())),
                    Err(e) => {
                        log::error!("[EMAIL-RESET] Database error: {:?}", e);
                        (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        )
                    }
                }
            } else {
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(
                        ErrorCode::EmailInvalid,
                        "This Email address hasn't been signed up.",
                    ))),
                )
            }
        }
        Err(e) => {
            log::error!("[EMAIL-RESET] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// User Sign-up
///
/// Sign up a user.
///
/// ## Parameters
///
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<RedisDb>`: Redis connection
/// - `Json<UserInfo>`: Json of UserInfo, including username, password, email, verification code
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `Json<UserResponse>`: Json of UserResponse, including burrow_id of user's assigned default burrow
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::EmailInvalid`
///   - `ErrorCode::EmailDuplicate`
///   - `ErrorCode::EmptyField`
///   - `ErrorCode::UsernameDuplicate`
///   - `ErrorCode::CredentialInvalid`
///   - `ErrorCode::DatabaseErr`
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
            .filter(db::user::Column::Email.eq(user.email))
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
            .filter(db::user::Column::Username.eq(user.username))
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
                                "Invalid verification code",
                            ))),
                        );
                    } else {
                        let delete_result: Result<i64, redis::RedisError> = redis::cmd("DEL")
                            .arg(&user.email)
                            .query_async(kvdb_con.as_mut())
                            .await;
                        match delete_result {
                            Ok(1) => {
                                log::info!("[SIGN-UP] delete email -> rate:code success");
                            }
                            Ok(_) => {
                                log::error!(
                                    "[SIGN-UP] delete zero or more than one email -> rate:code"
                                );
                                return (
                                    Status::InternalServerError,
                                    Err(Json(ErrorResponse::default())),
                                );
                            }
                            Err(e) => {
                                log::error!("[SIGN-UP] Database Error: {:?}", e);
                                return (
                                    Status::InternalServerError,
                                    Err(Json(ErrorResponse::default())),
                                );
                            }
                        }
                    }
                }
                None => {
                    return (
                        Status::BadRequest,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::CredentialInvalid,
                            "Invalid verification code",
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
        let users = db::user::ActiveModel {
            uid: Set(uid),
            username: Set(user.username.to_string()),
            password: Set(password),
            email: Set(user.email.to_string()),
            create_time: Set(now.to_owned()),
            salt: Set(salt),
        };

        let burrows = db::burrow::ActiveModel {
            uid: Set(uid),
            title: Set("Default".to_owned()),
            description: Set("".to_owned()),
            create_time: Set(now.to_owned()),
            update_time: Set(now.to_owned()),
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
                    let users_status = db::user_status::ActiveModel {
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

/// User Reset
///
/// User Resets password in logout status, requires verification code from verification email sent by `user_email_service`.
///
/// ## Parameters
///
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<RedisDb>`: Redis connection
/// - `CookieJar`: Collection of Cookie
/// - `Json<UserResetInfo>`: Json of UserResetInfo, including password, email, verification code
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: String "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::EmailInvalid`
///   - `ErrorCode::CredentialInvalid`
///   - `ErrorCode::DatabaseErr`
#[post("/reset", data = "<user_info>", format = "json")]
pub async fn user_reset(
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    cookies: &CookieJar<'_>,
    user_info: Json<UserResetInfo<'_>>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    let mut kv_conn = kvdb.into_inner();
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
    } else {
        // check if email address is in use
        let user_stored = match User::find()
            .filter(db::user::Column::Email.eq(user.email))
            .one(&pg_con)
            .await
        {
            Ok(res) => match res {
                None => {
                    return (
                        Status::BadRequest,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::EmailInvalid,
                            "This Email address hasn't been signed up.",
                        ))),
                    );
                }
                Some(u) => u,
            },
            Err(e) => {
                log::error!("[RESET] Database Error: {:?}", e);
                return (
                    Status::InternalServerError,
                    Err(Json(ErrorResponse::default())),
                );
            }
        };
        // check if verification code is correct, return corresponding error if so
        let get_redis_result: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
            .arg(&user.email)
            .query_async(kv_conn.as_mut())
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
                                "Invalid verification code",
                            ))),
                        );
                    } else {
                        let delete_result: Result<i64, redis::RedisError> = redis::cmd("DEL")
                            .arg(&user.email)
                            .query_async(kv_conn.as_mut())
                            .await;
                        match delete_result {
                            Ok(1) => {
                                log::info!("[RESET] delete email -> rate:code success");
                            }
                            Ok(_) => {
                                log::error!(
                                    "[RESET] delete zero or more than one email -> rate:code"
                                );
                                return (
                                    Status::InternalServerError,
                                    Err(Json(ErrorResponse::default())),
                                );
                            }
                            Err(e) => {
                                log::error!("[RESET] Database Error: {:?}", e);
                                return (
                                    Status::InternalServerError,
                                    Err(Json(ErrorResponse::default())),
                                );
                            }
                        }
                    }
                }
                None => {
                    return (
                        Status::BadRequest,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::CredentialInvalid,
                            "Invalid verification code",
                        ))),
                    )
                }
            },
            Err(e) => {
                log::error!("[RESET] Database Error: {:?}", e);
                return (
                    Status::InternalServerError,
                    Err(Json(ErrorResponse::default())),
                );
            }
        };

        // get salt
        let salt = user_stored.salt.clone();
        let uid = user_stored.uid;
        // encrypt password
        let mut hash_sha3 = Sha3::sha3_256();
        hash_sha3.input_str(&(salt + user.password));
        let password = hash_sha3.result_str();
        let mut users: db::user::ActiveModel = user_stored.into();
        users.password = Set(password);
        // insert rows in database
        match users.update(&pg_con).await {
            Ok(_) => {
                let token = match crate::utils::auth::set_token(uid, kv_conn.as_mut()).await {
                    Ok(t) => t,
                    Err(e) => return (Status::InternalServerError, Err(Json(e))),
                };
                // build cookie
                let cookie = Cookie::build("token", token).cookie_options().finish();
                // set cookie
                cookies.add_private(cookie);
                info!("[RESET] User login complete.");
                (Status::Ok, Ok("Success".to_string()))
            }
            Err(e) => {
                error!("[RESET] Database error: {:?}", e);
                (
                    Status::InternalServerError,
                    Err(Json(ErrorResponse::default())),
                )
            }
        }
    }
}

/// User Change Password
///
/// User changes password in login status, no requirement for verification code.
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<RedisDb>`: Redis connection
/// - `CookieJar`: Collection of Cookie
/// - `Json<UserChangePassword>`: Json of UserChangePassword, including old password, new password
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: String "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::UserNotExist`
///   - `ErrorCode::CredentialInvalid`
///   - `ErrorCode::DatabaseErr`
#[post("/change", data = "<user_info>", format = "json")]
pub async fn user_change_password(
    auth: Auth,
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    cookies: &CookieJar<'_>,
    user_info: Json<UserChangePassword<'_>>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    let mut kv_conn = kvdb.into_inner();
    // get user info from request
    let user = user_info.into_inner();
    // check if email address is valid, add corresponding error if so
    // check if email address is in use
    let user_stored = match User::find_by_id(auth.id).one(&pg_con).await {
        Ok(res) => match res {
            None => {
                return (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(
                        ErrorCode::UserNotExist,
                        "User not exist.",
                    ))),
                );
            }
            Some(u) => u,
        },
        Err(e) => {
            log::error!("[RESET] Database Error: {:?}", e);
            return (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            );
        }
    };
    // get salt
    let salt = user_stored.salt.clone();
    let uid = user_stored.uid;
    let old_password = user_stored.password.clone();
    // encrypt password
    let mut hash_sha3 = Sha3::sha3_256();
    hash_sha3.input_str(&(salt.clone() + user.password));
    let password = hash_sha3.result_str();
    if password != old_password {
        return (
            Status::BadRequest,
            Err(Json(ErrorResponse::build(
                ErrorCode::CredentialInvalid,
                "Wrong password.",
            ))),
        );
    }
    let mut hash_sha3 = Sha3::sha3_256();
    hash_sha3.input_str(&(salt + user.new_password));
    let new_password = hash_sha3.result_str();
    let mut users: db::user::ActiveModel = user_stored.into();
    users.password = Set(new_password);
    // insert rows in database
    match users.update(&pg_con).await {
        Ok(_) => {
            let token = match crate::utils::auth::set_token(uid, kv_conn.as_mut()).await {
                Ok(t) => t,
                Err(e) => return (Status::InternalServerError, Err(Json(e))),
            };
            // build cookie
            let cookie = Cookie::build("token", token).cookie_options().finish();
            // set cookie
            cookies.add_private(cookie);
            info!("[RESET] User login complete.");
            (Status::Ok, Ok("Success".to_string()))
        }
        Err(e) => {
            error!("[RESET] Database error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// User Log in
///
/// Log in a user.
///
/// ## Parameters
///
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<RedisDb>`: Redis connection
/// - `CookieJar`: Collection of Cookie
/// - `Json<UserLoginInfo>`: Json of UserLoginInfo, including username, password
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: String "Success"
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::CredentialInvalid`
///   - `ErrorCode::DatabaseErr`
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
        .filter(db::user::Column::Username.eq(user.username))
        .one(&db.into_inner())
        .await
    {
        Ok(s) => match s {
            Some(matched_user) => {
                info!("[LOGIN] username exists, continue...");
                let salt = matched_user.salt;
                // if salt.is_empty() {
                //     error!("[LOGIN] cannot find user's salt.");
                //     return (
                //         Status::InternalServerError,
                //         Err(Json(ErrorResponse::default())),
                //     );
                // }
                // encrypt input password same as sign-up
                let mut hash_sha3 = Sha3::sha3_256();
                hash_sha3.input_str(&(salt + user.password));
                let password = hash_sha3.result_str();
                // check if password is wrong, add corresponding error if so
                if matched_user.password.eq(&password) {
                    info!("[LOGIN] password correct, continue...");
                    let token = match set_token(matched_user.uid, con.as_mut()).await {
                        Ok(t) => t,
                        Err(e) => return (Status::InternalServerError, Err(Json(e))),
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

/// User logout
///
/// Logout a user.
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<RedisDb>`: Redis connection
/// - `CookieJar`: Collection of Cookie
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
#[get("/logout")]
pub async fn user_logout(
    auth: Auth,
    kvdb: Connection<RedisDb>,
    cookies: &CookieJar<'_>,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    let mut kv_conn = kvdb.into_inner();
    // get user info from request
    let uid = auth.id;
    match delete_token(uid, kv_conn.as_mut()).await {
        Ok(_) => {
            let delete_result: Result<i64, redis::RedisError> = redis::cmd("DEL")
                .arg(uid)
                .query_async(kv_conn.as_mut())
                .await;
            match delete_result {
                Ok(1) => info!("[TOKEN] delete id->token"),
                Ok(0) => info!("[TOKEN] no id->token found"),
                Ok(_) => {
                    error!("[TOKEN] failed to delete refresh_token -> id when login.");
                    return (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    );
                }
                Err(e) => {
                    error!(
                        "[TOKEN] failed to delete token -> id when login. RedisError: {:?}",
                        e
                    );
                    return (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::default())),
                    );
                }
            };
            let mut cookie = Cookie::named("token");
            cookie.set_domain(".thuburrow.com");
            cookies.remove_private(cookie);
            (Status::Ok, Ok("Success".to_string()))
        }
        Err(e) => {
            error!("[LOGOUT] Database error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// Get Burrow
///
/// Show burrows that belongs to current user, discarded burrows won't be shown here.
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `Json<Vec<BurrowMetadata>>`: Json of Vec<BurrowMetadata>, including id, title, description, amount-of-post of selected burrows
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::UserNotExist`
///   - `ErrorCode::DatabaseErr`
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
    match db::user_status::Entity::find_by_id(auth.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_state) => match opt_state {
            Some(state) => {
                let valid_burrows = get_burrow_list(&state.valid_burrow);
                let banned_burrows = get_burrow_list(&state.banned_burrow);
                let burrow_ids = [valid_burrows, banned_burrows].concat();
                match Burrow::find()
                    .filter(Condition::any().add(db::burrow::Column::BurrowId.is_in(burrow_ids)))
                    .order_by_desc(db::burrow::Column::BurrowId)
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

/// Get Collection
///
/// Show posts in user's collection.
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `Option<usize>`: page number, default value 0
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `Json<Vec<UserGetCollectionResponse>>`: Json of Vec<UserGetCollectionResponse>, including struct `Post`, a bool showing if the posts in collection are updated.
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::DatabaseErr`
#[get("/collection?<page>")]
pub async fn get_collection(
    db: Connection<PgDb>,
    auth: Auth,
    page: Option<usize>,
) -> (
    Status,
    Result<Json<Vec<UserGetCollectionResponse>>, Json<ErrorResponse>>,
) {
    let pg_con = db.into_inner();
    let page = page.unwrap_or(0);
    match UserCollection::find()
        .filter(db::user_collection::Column::Uid.eq(auth.id))
        .order_by_desc(db::user_collection::Column::PostId)
        .paginate(&pg_con, POST_PER_PAGE)
        .fetch_page(page)
        .await
    {
        Ok(results) => {
            let post_ids = results.iter().map(|r| r.post_id).collect::<Vec<i64>>();
            match ContentPost::find()
                .filter(db::content_post::Column::PostId.is_in(post_ids))
                .order_by_desc(db::content_post::Column::PostId)
                .all(&pg_con)
                .await
            {
                Ok(posts) => {
                    if posts.len() == results.len() {
                        (
                            Status::Ok,
                            Ok(Json(
                                posts
                                    .iter()
                                    .map(|post| {
                                        let p: Post = post.into();
                                        p
                                    })
                                    .zip(results.iter().map(|r| r.is_update))
                                    .map(|(post, is_update)| UserGetCollectionResponse {
                                        post,
                                        is_update,
                                    })
                                    .collect(),
                            )),
                        )
                    } else {
                        let hm: HashMap<i64, bool> =
                            results.iter().map(|r| (r.post_id, r.is_update)).collect();
                        (
                            Status::Ok,
                            Ok(Json(
                                posts
                                    .iter()
                                    .map(|p| {
                                        let post: Post = p.into();
                                        let is_update = *hm.get(&post.post_id).unwrap_or(&false);
                                        UserGetCollectionResponse { post, is_update }
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
    // match ContentPost::find()
    //     .filter(
    //         Condition::any().add(
    //             pgdb::content_post::Column::PostId.in_subquery(
    //                 Query::select()
    //                     .column(pgdb::user_collection::Column::PostId)
    //                     .from(UserCollection)
    //                     .and_where(Expr::col(pgdb::user_collection::Column::Uid).eq(auth.id))
    //                     .order_by_columns(vec![(
    //                         pgdb::user_collection::Column::PostId,
    //                         Order::Desc,
    //                     )])
    //                     .limit(POST_PER_PAGE as u64)
    //                     .offset(page * POST_PER_PAGE as u64)
    //                     .to_owned(),
    //             ),
    //         ),
    //     )
    //     .order_by_desc(pgdb::content_post::Column::PostId)
    //     .all(&pg_con)
    //     .await
    // {
    //     Ok(posts) => (
    //         Status::Ok,
    //         Ok(Json(posts.iter().map(|post| post.into()).collect())),
    //     ),
    //     Err(e) => {
    //         error!("[GET-FAV] Database Error: {:?}", e);
    //         (
    //             Status::InternalServerError,
    //             Err(Json(ErrorResponse::default())),
    //         )
    //     }
    // }
}

/// Get Follow
///
/// Show burrows that current user follows.
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
/// - `Option<usize>`: page number, default value 0
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `Json<Vec<UserGetFollowResponse>>`: Json of Vec<UserGetFollowResponse>, including struct `BurrowMetadata`, a bool showing if user's followed burrows are updated.
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::DatabaseErr`
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
    match UserFollow::find()
        .filter(db::user_follow::Column::Uid.eq(auth.id))
        .order_by_desc(db::user_follow::Column::BurrowId)
        .paginate(&pg_con, BURROW_PER_PAGE)
        .fetch_page(page)
        .await
    {
        Ok(results) => {
            let burrow_ids = results.iter().map(|r| r.burrow_id).collect::<Vec<i64>>();
            match Burrow::find()
                .filter(db::burrow::Column::BurrowId.is_in(burrow_ids))
                .order_by_desc(db::burrow::Column::BurrowId)
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

/// Get User Valid Burrow
///
/// Get burrows that hasn't been banned or discarded.
///
/// ## Parameters
///
/// - `Auth`: Authenticated user
/// - `Connection<PgDb>`: Postgres connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `Json<Vec<i64>>`: Json of Vec<i64>, including struct `Post`, a bool showing if the posts in collection is updated.
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///   - `ErrorCode::UserNotExist`
///   - `ErrorCode::DatabaseErr`
#[get("/valid-burrows")]
pub async fn get_user_valid_burrow(
    auth: Auth,
    db: Connection<PgDb>,
) -> (Status, Result<Json<Vec<i64>>, Json<ErrorResponse>>) {
    let pg_con = db.into_inner();
    match UserStatus::find_by_id(auth.id).one(&pg_con).await {
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
