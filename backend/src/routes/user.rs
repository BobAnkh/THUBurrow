use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
// use rocket::response::status;
use rocket::http::Status;
use rocket_db_pools::Connection;
use sea_orm::entity::*;
use sea_orm::QueryFilter;

use crate::pgdb;
use crate::pgdb::user::Entity as User;
use crate::pool::{PgDb, RedisDb};
use crate::req::user::*;

use crypto::digest::Digest;
use crypto::sha3::Sha3;
use std::iter;

use idgenerator::IdHelper;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/users", routes![user_log_in, user_sign_up])
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
) -> (Status, Option<Json<UserResponse>>) {
    // create vec of errors
    let mut error_collector = Vec::new();
    // get user info from request
    let user = user_info.into_inner();
    // check if email address is valid, add corresponding error if so
    if !user.email.ends_with("tsinghua.edu.cn") {
        error_collector.push("Illegal Email Address".to_string());
    }
    // check if email address is duplicated, add corresponding error if so
    let email_dup_result = User::find()
        .filter(pgdb::user::Column::Email.eq(user.email))
        .one(&db)
        .await;
    match email_dup_result {
        Ok(res) => {
            if res.is_some() {
                error_collector.push("Duplicated Email Address".to_string());
            }
        }
        _ => return (Status::InternalServerError, None),
    }
    // check if username is duplicated, add corresponding error if so
    let username_dup_result = User::find()
        .filter(pgdb::user::Column::Username.eq(user.username))
        .one(&db)
        .await;
    match username_dup_result {
        Ok(res) => {
            if res.is_some() {
                error_collector.push("Duplicated Username".to_string());
            }
        }
        _ => return (Status::InternalServerError, None),
    }
    // if error exists, refuse to add user
    if !error_collector.is_empty() {
        let user_response = UserResponse {
            errors: error_collector,
        };
        (Status::BadRequest, Some(Json(user_response)))
    } else {
        // generate salt
        let salt = gen_salt().await;
        // encrypt password
        let mut hash_sha3 = Sha3::sha3_256();
        hash_sha3.input_str(&(String::from(&salt) + user.password));
        let password = hash_sha3.result_str();
        // generate uid
        let uid: i64 = IdHelper::next_id();
        // fill the row
        let users = pgdb::user::ActiveModel {
            uid: Set(uid.to_owned()),
            username: Set(Some(user.username.to_string())),
            password: Set(Some(password)),
            email: Set(Some(user.email.to_string())),
            salt: Set(Some(salt.to_string())),
            ..Default::default()
        };
        // insert the row in database
        let ins_result = users.insert(&db).await;
        match ins_result {
            Ok(res) => {
                info!("User signup Succ, save user: {:?}", res.uid);
                let user_response = UserResponse {
                    errors: error_collector,
                };
                (Status::Ok, Some(Json(user_response)))
            }
            _ => (Status::InternalServerError, None),
        }
    }
}

#[post("/login", data = "<user_info>", format = "json")]
pub async fn user_log_in(
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    cookies: &CookieJar<'_>,
    user_info: Json<UserLoginInfo<'_>>,
) -> (Status, Option<Json<UserResponse>>) {
    let mut con = kvdb.into_inner();
    // create a response struct
    let mut user_response = UserResponse { errors: Vec::new() };
    // get user info from request
    let user = user_info.into_inner();
    // check if username is existed, add corresponding error if so
    let username_existence_result = User::find()
        .filter(pgdb::user::Column::Username.eq(user.username))
        .one(&db)
        .await;
    // check if password is wrong, add corresponding error if so
    match username_existence_result {
        Ok(s) => match s {
            Some(matched_user) => {
                info!("username exists, continue...");
                let salt = match matched_user.salt {
                    Some(s) => s,
                    None => {
                        error!("cannot find user's salt.");
                        return (Status::InternalServerError, None);
                    }
                };
                // encrypt input password same as sign-up
                let mut hash_sha3 = Sha3::sha3_256();
                hash_sha3.input_str(&(String::from(&salt) + user.password));
                let password = hash_sha3.result_str();
                if matched_user.password.eq(&Some(password.to_string())) {
                    info!("password correct, continue...");
                    // find old token by get id
                    let old_token_get: Result<String, redis::RedisError> = redis::cmd("GET")
                        .arg(matched_user.uid)
                        .query_async(con.as_mut())
                        .await;
                    // if old token -> id exists
                    if let Ok(old_token) = old_token_get {
                        info!("find old token:{:?}, continue...", old_token);
                        // clear old token -> id
                        let delete_result: Result<i64, redis::RedisError> = redis::cmd("DEL")
                            .arg(&old_token)
                            .query_async(con.as_mut())
                            .await;
                        match delete_result {
                            Ok(1) => info!("delete token->id"),
                            Ok(0) => info!("no ref_token->id found"),
                            _ => {
                                error!("failed to delete token -> id when login.");
                                return (Status::InternalServerError, None);
                            }
                        };
                        // find old refresh_token by hashing old token
                        let mut hash_sha3 = Sha3::sha3_384();
                        hash_sha3.input_str(&old_token);
                        let old_refresh_token = hash_sha3.result_str();
                        // clear old refresh_token -> id
                        let delete_result: Result<i64, redis::RedisError> = redis::cmd("DEL")
                            .arg(&old_refresh_token)
                            .query_async(con.as_mut())
                            .await;
                        match delete_result {
                            Ok(1) => info!("delete ref_token->id"),
                            Ok(0) => info!("no ref_token->id found"),
                            _ => {
                                error!("failed to delete ref_token -> id when login.");
                                return (Status::InternalServerError, None);
                            }
                        };
                    };
                    // generate token and refresh token
                    let token: String = iter::repeat(())
                        .map(|()| thread_rng().sample(Alphanumeric))
                        .map(char::from)
                        .take(32)
                        .collect();
                    let mut hash_sha3 = Sha3::sha3_384();
                    hash_sha3.input_str(&token);
                    let refresh_token = hash_sha3.result_str();
                    // build cookie
                    let cookie = Cookie::build("token", token.clone())
                        .domain("thuburrow.com")
                        .path("/")
                        .same_site(SameSite::None)
                        .finish();
                    // set cookie
                    cookies.add_private(cookie);
                    // set token -> id
                    let uid_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                        .arg(&token)
                        // .arg(4 * 3600)
                        .arg(15)
                        .arg(matched_user.uid)
                        .query_async(con.as_mut())
                        .await;
                    match uid_result {
                        Ok(s) => info!("setex token->id: {:?} -> {}", &token, s),
                        _ => {
                            error!("failed to set token -> id when login.");
                            return (Status::InternalServerError, None);
                        }
                    };
                    // set refresh_token -> id
                    let uid_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                        .arg(&refresh_token)
                        // .arg(15 * 24 * 3600)
                        .arg(30)
                        .arg(matched_user.uid)
                        .query_async(con.as_mut())
                        .await;
                    match uid_result {
                        Ok(s) => info!("setex refresh_token->id: {:?} -> {}", &refresh_token, s),
                        _ => {
                            error!("failed to set refresh_token -> id when login.");
                            return (Status::InternalServerError, None);
                        }
                    };
                    // set id -> token
                    let token_result: Result<String, redis::RedisError> = redis::cmd("SET")
                        .arg(matched_user.uid)
                        .arg(&token)
                        .query_async(con.as_mut())
                        .await;
                    match token_result {
                        Ok(s) => info!("set id->token: {} -> {:?}", matched_user.uid, s),
                        _ => {
                            error!("failed to set id -> token when login.");
                            return (Status::InternalServerError, None);
                        }
                    };
                    info!("User login complete.");
                    (Status::Ok, Some(Json(user_response)))
                } else {
                    info!("wrong password.");
                    user_response.errors.push("Wrong password".to_string());
                    (Status::BadRequest, Some(Json(user_response)))
                }
            }
            None => {
                info!("username does not exists.");
                user_response
                    .errors
                    .push("Username does not exist".to_string());
                (Status::BadRequest, Some(Json(user_response)))
            }
        },
        _ => (Status::InternalServerError, None),
    }
}
