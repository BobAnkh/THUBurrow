use chrono::{FixedOffset, Utc};
// use futures::future;
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::sea_query::{Expr, Query};
use sea_orm::{entity::*, query::*};
use sea_orm::{DbErr, QueryFilter};
// , DatabaseConnection};

use crate::pgdb;
use crate::pgdb::prelude::*;
use crate::pool::{PgDb, PulsarSearchProducerMq, RedisDb, RocketPulsarProducer};
use crate::req::pulsar::*;
use crate::req::{
    burrow::{BurrowMetadata, BURROW_PER_PAGE},
    content::{Post, POST_PER_PAGE},
    user::*,
};
use crate::utils::auth::Auth;
use crate::utils::burrow_valid::*;
use crate::utils::email;

use crypto::digest::Digest;
use crypto::sha3::Sha3;
use std::collections::HashMap;
use std::iter;

use idgenerator::IdHelper;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

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
    pulsar: Connection<PulsarSearchProducerMq>,
    relation_info: Json<RelationData>,
) -> Status {
    let relation = relation_info.into_inner();
    let msg = relation.to_pulsar(auth.id);
    let mut producer = match pulsar
        .get_producer("persistent://public/default/relation")
        .await
    {
        Ok(producer) => producer,
        Err(e) => {
            log::error!("{}", e);
            return Status::InternalServerError;
        }
    };
    match producer.send(msg).await {
        Ok(_) => log::info!("send data to pulsar successfully!"),
        Err(e) => {
            log::error!("Err: {}", e);
            return Status::InternalServerError;
        }
    }
    Status::Ok
}

#[post("/sign-up", data = "<user_info>", format = "json")]
pub async fn user_sign_up(
    db: Connection<PgDb>,
    user_info: Json<UserInfo<'_>>,
) -> (Status, Json<UserResponse>) {
    let pg_con = db.into_inner();
    // create vec of errors
    let mut errors = Vec::new();
    // get user info from request
    let user = user_info.into_inner();
    // check if email address is valid, add corresponding error if so
    if !email::check_email_syntax(user.email) {
        return (
            Status::BadRequest,
            Json(UserResponse {
                default_burrow: -1,
                errors: vec!["Illegal Email Address".to_string()],
            }),
        );
    }
    // check if email address is duplicated, add corresponding error if so
    match User::find()
        .filter(pgdb::user::Column::Email.eq(user.email))
        .one(&pg_con)
        .await
    {
        Ok(res) => {
            if res.is_some() {
                errors.push("Duplicated Email Address".to_string());
            }
        }
        Err(e) => {
            log::error!("[SIGN-UP] Database Error: {:?}", e);
            return (
                Status::InternalServerError,
                Json(UserResponse {
                    default_burrow: -1,
                    errors: Vec::new(),
                }),
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
                errors.push("Duplicated Username".to_string());
            }
        }
        Err(e) => {
            log::error!("[SIGN-UP] Database Error: {:?}", e);
            return (
                Status::InternalServerError,
                Json(UserResponse {
                    default_burrow: -1,
                    errors: Vec::new(),
                }),
            );
        }
    }
    // if error exists, refuse to add user
    if !errors.is_empty() {
        (
            Status::BadRequest,
            Json(UserResponse {
                default_burrow: -1,
                errors,
            }),
        )
    } else {
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
            Ok(default_burrow) => (
                Status::Ok,
                Json(UserResponse {
                    default_burrow,
                    errors,
                }),
            ),
            Err(e) => {
                error!("[SIGN-UP] Database error: {:?}", e);
                (
                    Status::InternalServerError,
                    Json(UserResponse {
                        default_burrow: -1,
                        errors: Vec::new(),
                    }),
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
) -> (Status, String) {
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
                    return (Status::InternalServerError, "".to_string());
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
                        _ => {
                            error!("[LOGIN] failed to set token -> id when login.");
                            return (Status::InternalServerError, "".to_string());
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
                        _ => {
                            error!("[LOGIN] failed to set refresh_token -> id when login.");
                            return (Status::InternalServerError, "".to_string());
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
                                    _ => {
                                        error!("[LOGIN] failed to delete token -> id when login.");
                                        return (Status::InternalServerError, "".to_string());
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
                                    _ => {
                                        error!(
                                            "[LOGIN] failed to delete ref_token -> id when login."
                                        );
                                        return (Status::InternalServerError, "".to_string());
                                    }
                                };
                                info!("[LOGIN] set id->token: {} -> {:?}", matched_user.uid, token);
                            }
                            None => {
                                info!("[LOGIN] no id->token found");
                                info!("[LOGIN] set id->token: {} -> {:?}", matched_user.uid, token);
                            }
                        },
                        _ => {
                            error!("[LOGIN] failed to set id -> token when login.");
                            return (Status::InternalServerError, "".to_string());
                        }
                    };
                    // build cookie
                    let cookie = Cookie::build("token", token)
                        .domain(".thuburrow.com")
                        .path("/")
                        .same_site(SameSite::Strict)
                        .secure(true)
                        .http_only(true)
                        .max_age(time::Duration::weeks(1))
                        .finish();
                    // set cookie
                    cookies.add_private(cookie);
                    info!("[LOGIN] User login complete.");
                    (Status::Ok, "".to_string())
                } else {
                    info!("[LOGIN] wrong password.");
                    (Status::BadRequest, "Wrong username or password".to_string())
                }
            }
            None => {
                info!("[LOGIN] username does not exists.");
                (Status::BadRequest, "Wrong username or password".to_string())
            }
        },
        Err(e) => {
            error!("[LOGIN] Database error: {:?}", e);
            (Status::InternalServerError, "".to_string())
        }
    }
}

#[get("/burrow")]
pub async fn get_burrow(db: Connection<PgDb>, auth: Auth) -> (Status, Json<Vec<BurrowMetadata>>) {
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
                        Json(burrows.iter().map(|burrow| burrow.into()).collect()),
                    ),
                    Err(e) => {
                        error!("[GET_BURROW] failed to get burrow list: {:?}", e);
                        (Status::InternalServerError, Json(Vec::new()))
                    }
                }
            }
            None => {
                info!("[GET-BURROW] Cannot find user_status by uid.");
                (Status::Forbidden, Json(Vec::new()))
            }
        },
        Err(e) => {
            error!("[GET-BURROW] Database Error: {:?}", e);
            (Status::InternalServerError, Json(Vec::new()))
        }
    }
}

#[get("/collection?<page>")]
pub async fn get_collection(
    db: Connection<PgDb>,
    auth: Auth,
    page: Option<u64>,
) -> (Status, Json<Vec<Post>>) {
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
            Json(posts.iter().map(|post| post.into()).collect()),
        ),
        Err(e) => {
            error!("[GET-FAV] Database Error: {:?}", e);
            (Status::InternalServerError, Json(Vec::new()))
        }
    }
}

#[get("/follow?<page>")]
pub async fn get_follow(
    db: Connection<PgDb>,
    auth: Auth,
    page: Option<usize>,
) -> (Status, Json<Vec<UserGetFollowResponse>>) {
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
                            Json(
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
                            ),
                        )
                    } else {
                        let hm: HashMap<i64, bool> =
                            results.iter().map(|r| (r.burrow_id, r.is_update)).collect();
                        (
                            Status::Ok,
                            Json(
                                burrows
                                    .iter()
                                    .map(|meta| {
                                        let burrow: BurrowMetadata = meta.into();
                                        let is_update =
                                            *hm.get(&burrow.burrow_id).unwrap_or(&false);
                                        UserGetFollowResponse { burrow, is_update }
                                    })
                                    .collect(),
                            ),
                        )
                    }
                }
                Err(e) => {
                    error!("[GET-FOLLOW] DataBase Error: {:?}", e.to_string());
                    (Status::InternalServerError, Json(Vec::new()))
                }
            }
        }
        Err(e) => {
            error!("[GET-FOLLOW] Database Error: {:?}", e.to_string());
            (Status::InternalServerError, Json(Vec::new()))
        }
    }
}

#[get("/valid-burrow")]
pub async fn get_user_valid_burrow(auth: Auth, db: Connection<PgDb>) -> (Status, Json<Vec<i64>>) {
    let pg_con = db.into_inner();
    match pgdb::user_status::Entity::find_by_id(auth.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_state) => match opt_state {
            Some(state) => (Status::Ok, Json(get_burrow_list(&state.valid_burrow))),
            None => {
                info!("[GET-VALID-BURROW] Cannot find user_status by uid.");
                (Status::Forbidden, Json(Vec::new()))
            }
        },
        Err(e) => {
            error!("[GET-VALID-BURROW] Database Error: {:?}", e);
            (Status::InternalServerError, Json(Vec::new()))
        }
    }
}
