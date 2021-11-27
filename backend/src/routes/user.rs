use chrono::{FixedOffset, Utc};
// use futures::future;
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::entity::*;
use sea_orm::QueryFilter;
// , DatabaseConnection};

use crate::pgdb;
use crate::pgdb::user::Entity as User;
use crate::pool::{PgDb, RedisDb};
use crate::req::user::*;
use crate::utils::email;
use crate::utils::get_valid_burrow::*;
use crate::utils::sso;

use crypto::digest::Digest;
use crypto::sha3::Sha3;
use std::iter;
use std::ops::Deref;

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
            follow_burrow,
            get_favorite,
            get_burrow
        ],
    )
}

pub async fn gen_salt() -> String {
    let salt: String = iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(8)
        .collect();
    salt
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
        _ => {
            return (
                Status::InternalServerError,
                Json(UserResponse { errors: Vec::new() }),
            )
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
        _ => {
            return (
                Status::InternalServerError,
                Json(UserResponse { errors: Vec::new() }),
            )
        }
    }
    // if error exists, refuse to add user
    if !errors.is_empty() {
        (Status::BadRequest, Json(UserResponse { errors }))
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
        let create_time = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
        let users = pgdb::user::ActiveModel {
            uid: Set(uid.to_owned()),
            username: Set(user.username.to_string()),
            password: Set(password),
            email: Set(user.email.to_string()),
            create_time: Set(create_time),
            salt: Set(salt),
        };
        let users_status = pgdb::user_status::ActiveModel {
            uid: Set(uid.to_owned()),
            update_time: Set(create_time),
            ..Default::default()
        };
        // insert rows in database
        let ins_user_result = users.insert(&pg_con).await;
        match ins_user_result {
            Ok(_) => {
                let ins_user_status_result = users_status.insert(&pg_con).await;
                match ins_user_status_result {
                    Ok(res) => {
                        info!(
                            "[SIGN-UP] User signup Succ, save user: {}",
                            res.uid.unwrap()
                        );
                        (Status::Ok, Json(UserResponse { errors }))
                    }
                    Err(e) => {
                        error!("[SIGN-UP] Database error: {:?}", e.to_string());
                        (
                            Status::InternalServerError,
                            Json(UserResponse { errors: Vec::new() }),
                        )
                    }
                }
            }
            _ => (
                Status::InternalServerError,
                Json(UserResponse { errors: Vec::new() }),
            ),
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
        _ => (Status::InternalServerError, "".to_string()),
    }
}

#[get("/burrow")]
pub async fn get_burrow(
    db: Connection<PgDb>,
    sso: sso::SsoAuth,
) -> (Status, Json<Vec<UserGetBurrowResponse>>) {
    // Ok(burrows) => {
    //     // let mut posts_num = Vec::new();
    //     let r: Vec<i64> = future::try_join_all(burrows.iter().map(move |burrow| {
    //         let inner_conn = pg_con.clone();
    //         GetBurrow::get_post(burrow, inner_conn)
    //     }))
    //     .await
    //     .unwrap();
    let pg_con = db.into_inner();
    match pgdb::user_status::Entity::find_by_id(sso.id)
        .one(&pg_con)
        .await
    {
        Ok(opt_state) => match opt_state {
            Some(state) => {
                let valid_burrows = match get_burrow_list(state.valid_burrow.clone()).await {
                    Ok(burrows_id) => burrows_id,
                    Err(e) => {
                        error!(
                            "[GET BURROW] Failed to get valid burrows: {:?}",
                            e.to_string()
                        );
                        return (Status::InternalServerError, Json(Vec::new()));
                    }
                };
                let banned_burrows = match get_burrow_list(state.banned_burrow.clone()).await {
                    Ok(burrows_id) => burrows_id,
                    Err(e) => {
                        error!(
                            "[GET BURROW] Failed to get valid burrows: {:?}",
                            e.to_string()
                        );
                        return (Status::InternalServerError, Json(Vec::new()));
                    }
                };
                let burrows_id = [valid_burrows, banned_burrows].concat();
                let mut response = Vec::new();
                for burrow_id in burrows_id {
                    match pgdb::burrow::Entity::find_by_id(burrow_id)
                        .one(&pg_con)
                        .await
                    {
                        Ok(opt_burrow) => match opt_burrow {
                            Some(burrow) => {
                                response.push(UserGetBurrowResponse {
                                    id: burrow.burrow_id,
                                    title: burrow.title.clone(),
                                    description: burrow.description.clone(),
                                    post_num: burrow.post_num,
                                });
                            }
                            None => {
                                error!("[GET-BURROW] Cannot find burrow by burrow_id.");
                                return (Status::InternalServerError, Json(Vec::new()));
                            }
                        },
                        Err(e) => {
                            error!("[GET-BURROW] Database Error: {:?}", e.to_string());
                            return (Status::InternalServerError, Json(Vec::new()));
                        }
                    };
                }
                (Status::Ok, Json(response))
            }
            None => {
                error!("[GET BURROW] Cannot find user_status by uid.");
                return (Status::InternalServerError, Json(Vec::new()));
            }
        },
        Err(e) => {
            error!("[GET BURROW] Database Error: {:?}", e.to_string());
            return (Status::InternalServerError, Json(Vec::new()));
        }
    }
}

#[get("/favorite")]
pub async fn get_favorite(
    db: Connection<PgDb>,
    sso: sso::SsoAuth,
) -> (Status, Json<Vec<UserGetFavResponse>>) {
    let pg_con = db.into_inner();
    match pgdb::user_like::Entity::find()
        .filter(pgdb::user_like::Column::Uid.eq(sso.id))
        .all(&pg_con)
        .await
    {
        Ok(results) => {
            let mut posts = Vec::new();
            for result in &results {
                match pgdb::content_post::Entity::find()
                    .filter(pgdb::content_post::Column::PostId.eq(result.post_id))
                    .one(&pg_con)
                    .await
                {
                    Ok(opt_post) => match opt_post {
                        Some(post) => {
                            posts.push(post);
                        }
                        None => {
                            error!("[GET-FAV] Database Error: cannot find post in table content_post by post_id from table user_like");
                            return (Status::InternalServerError, Json(Vec::new()));
                        }
                    },
                    Err(e) => {
                        error!("[GET-FAV] DataBase Error: {:?}", e.to_string());
                        return (Status::InternalServerError, Json(Vec::new()));
                    }
                };
            }
            // find burrow title by burrow_id
            let mut burrow_titles = Vec::new();
            for post in &posts {
                match pgdb::burrow::Entity::find_by_id(post.post_id)
                    .one(&pg_con)
                    .await
                {
                    Ok(opt_burrow) => match opt_burrow {
                        Some(burrow) => {
                            burrow_titles.push(burrow.title);
                        }
                        None => {
                            error!("[GET-FAV] Database Error: cannot find post in table content_post by post_id from table user_like");
                            return (Status::InternalServerError, Json(Vec::new()));
                        }
                    },
                    Err(e) => {
                        error!("[GET-FAV] Database Error: {:?}", e.to_string());
                        return (Status::InternalServerError, Json(Vec::new()));
                    }
                };
            }
            let mut burrow_titles = burrow_titles.iter();
            (
                Status::Ok,
                Json(
                    posts
                        .iter()
                        .map(|x| UserGetFavResponse {
                            post_id: x.post_id,
                            title: x.title.clone(),
                            tags: x.tag.clone(),
                            burrow_id: x.burrow_id,
                            burrow_name: burrow_titles.next().unwrap().deref().to_string(),
                        })
                        .collect(),
                ),
            )
        }
        Err(e) => {
            error!("[GET-FAV] Database Error: {:?}", e.to_string());
            return (Status::InternalServerError, Json(Vec::new()));
        }
    }
}

#[get("/follow")]
pub async fn get_follow(
    db: Connection<PgDb>,
    sso: sso::SsoAuth,
) -> (Status, Json<Vec<UserGetFollowResponse>>) {
    let pg_con = db.into_inner();
    match pgdb::user::Entity::find_by_id(sso.id).one(&pg_con).await {
        Ok(user) => {
            let user = user.unwrap();
            match user.find_related(pgdb::burrow::Entity).all(&pg_con).await {
                Ok(burrows) => {
                    (
                        Status::Ok,
                        Json(
                            burrows
                                .iter()
                                .map(|x| UserGetFollowResponse {
                                    id: x.burrow_id,
                                    title: x.title.clone(),
                                    description: x.description.clone(),
                                    // TODO
                                    update: false,
                                })
                                .collect(),
                        ),
                    )
                }
                Err(e) => {
                    error!("[GET-FOLLOW] Database Error: {:?}", e.to_string());
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

#[post("/follow/<burrow_id>")]
pub async fn follow_burrow(
    burrow_id: i64,
    db: Connection<PgDb>,
    sso: sso::SsoAuth,
) -> (Status, Json<String>) {
    let pg_con = db.into_inner();
    let uid = sso.id;
    let user_follow = pgdb::user_follow::ActiveModel {
        uid: Set(uid),
        burrow_id: Set(burrow_id),
        ..Default::default()
    };
    match user_follow.insert(&pg_con).await {
        Ok(res) => {
            info!(
                "[FOLLOW] User {} follows Burrow {}",
                res.uid.unwrap(),
                res.burrow_id.unwrap()
            );
            (Status::Ok, Json("".to_string()))
        }
        Err(e) => {
            error!("[FOLLOW] Database Error: {:?}", e.to_string());
            (Status::InternalServerError, Json("".to_string()))
        }
    }
}

#[post("/favorite/<post_id>")]
pub async fn like_post(
    post_id: i64,
    db: Connection<PgDb>,
    sso: sso::SsoAuth,
) -> (Status, Json<String>) {
    let pg_con = db.into_inner();
    let uid = sso.id;
    let user_like = pgdb::user_like::ActiveModel {
        uid: Set(uid),
        post_id: Set(post_id),
    };
    match user_like.insert(&pg_con).await {
        Ok(res) => {
            info!(
                "[LIKE] User {} likes Post {}",
                res.uid.unwrap(),
                res.post_id.unwrap()
            );
            (Status::Ok, Json("".to_string()))
        }
        Err(e) => {
            error!("[LIKE] Database Error: {:?}", e.to_string());
            (Status::InternalServerError, Json("".to_string()))
        }
    }
}
