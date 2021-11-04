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
use crate::utils::sso;

use crypto::digest::Digest;
use crypto::sha3::Sha3;
use std::iter;

use idgenerator::IdHelper;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/users", routes![user_log_in, user_sign_up, sso_test])
}

pub async fn gen_salt() -> String {
    let salt: String = iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(8)
        .collect();
    salt
}

#[get("/test/sso")]
pub async fn sso_test(a: sso::SsoAuth) -> Json<i64> {
    Json(a.id)
}

#[post("/sign-up", data = "<user_info>", format = "json")]
pub async fn user_sign_up(
    db: Connection<PgDb>,
    user_info: Json<UserInfo<'_>>,
) -> (Status, Json<UserSignupResponse>) {
    // create a response struct
    let mut signup_response = UserSignupResponse {
        success: false,
        errors: Vec::new(),
    };
    // get user info from request
    let user = user_info.into_inner();
    // check if email address is valid, add corresponding error if so
    if !user.email.ends_with("tsinghua.edu.cn"){
        signup_response
            .errors
            .push("Illegal Email Address".to_string());
    }
    // check if email address is duplicated, add corresponding error if so
    let email_dup_result = User::find()
        .filter(pgdb::user::Column::Email.eq(user.email))
        .one(&db)
        .await
        .expect("cannot fetch email data from pgdb");
    if let Some(_) = email_dup_result {
        signup_response
            .errors
            .push("Duplicated Email Address".to_string());
    }
    // check if username is duplicated, add corresponding error if so
    let username_dup_result = User::find()
        .filter(pgdb::user::Column::Username.eq(user.username))
        .one(&db)
        .await
        .expect("cannot fetch username data from pgdb");
    if let Some(_) = username_dup_result {
        signup_response
            .errors
            .push("Duplicated Username".to_string());
    }
    // if error exists, refuse to add user
    if !signup_response.errors.is_empty() {
        (Status::BadRequest, Json(signup_response))
    } else {
        signup_response.success = true;
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
            username: Set(Some(user.username.to_string()).to_owned()),
            password: Set(Some(password).to_owned()),
            email: Set(Some(user.email.to_string()).to_owned()),
            salt: Set(Some(salt.to_string().to_owned())),
            ..Default::default()
        };
        // insert the row in database
        let res = users.insert(&db).await.expect("Cannot save user");
        println!("{:?}", res.uid);
        // return the response
        (Status::Ok, Json(signup_response))
    }
}

#[post("/login", data = "<user_info>", format = "json")]
pub async fn user_log_in(
    db: Connection<PgDb>,
    kvdb: Connection<RedisDb>,
    cookies: &CookieJar<'_>,
    user_info: Json<UserLoginInfo<'_>>,
) -> (Status, Option<Json<UserLoginResponse>>) {
    let mut con = kvdb.into_inner();
    // create a response struct
    let mut login_response = UserLoginResponse {
        success: false,
        errors: Vec::new(),
    };
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
                let salt = match matched_user.salt {
                    Some(s) => s,
                    None => return (Status::BadRequest, None),
                };
                // encrypt input password same as sign-up
                let mut hash_sha3 = Sha3::sha3_256();
                hash_sha3.input_str(&(String::from(&salt) + user.password));
                let password = hash_sha3.result_str();
                if matched_user.password.eq(&Some(password.to_string())) {
                    login_response.success = true;
                    // find old token by get uid
                    let old_token_get: Result<String, redis::RedisError> = redis::cmd("GET")
                        .arg(matched_user.uid)
                        .query_async(con.as_mut())
                        .await;
                    // if old token -> uid exists
                    if let Ok(old_token) = old_token_get { 
                        println!("old token:{:?}", old_token);
                        // clear old token -> uid
                        let delete_result: Result<i64, redis::RedisError> = redis::cmd("DEL")
                            .arg(&old_token)
                            .query_async(con.as_mut())
                            .await;
                        match delete_result {
                            Ok(_) => println!("delete token->id"),
                            _ => return (Status::InternalServerError, None),
                        };
                        // find old refresh_token by hashing old token
                        let mut hash_sha3 = Sha3::sha3_384();
                        hash_sha3.input_str(&old_token);
                        let old_refresh_token = hash_sha3.result_str();
                        // clear old refresh_token -> uid
                        let delete_result: Result<i64, redis::RedisError> = redis::cmd("DEL")
                            .arg(&old_refresh_token)
                            .query_async(con.as_mut())
                            .await;
                        match delete_result {
                            Ok(_) => println!("delete ref_token->id"),
                            _ => return (Status::InternalServerError, None),
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
                    // set token -> uid
                    let uid_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                        .arg(&token)
                        .arg(4*3600)
                        .arg(matched_user.uid)
                        .query_async(con.as_mut())
                        .await;
                    match uid_result {
                        Ok(s) => println!("setex token->id: {:?} -> {}", &token, s),
                        _ => return (Status::InternalServerError, None),
                    };
                    // set refresh_token -> uid
                    let uid_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
                        .arg(&refresh_token)
                        .arg(15*24*3600)
                        .arg(matched_user.uid)
                        .query_async(con.as_mut())
                        .await;
                    match uid_result {
                        Ok(s) => println!("setex refresh_token->id: {:?} -> {}", &refresh_token, s),
                        _ => return (Status::InternalServerError, None),
                    };
                    // set uid -> token
                    let token_result: Result<String, redis::RedisError> = redis::cmd("SET")
                        .arg(matched_user.uid)
                        .arg(&token)
                        .query_async(con.as_mut())
                        .await;
                    match token_result {
                        Ok(s) => println!("set id->token: {} -> {:?}", matched_user.uid, s),
                        _ => return (Status::InternalServerError, None),
                    };
                    (Status::Ok, Some(Json(login_response)))
                }
                else {
                    login_response.errors.push("Wrong password".to_string());
                    (Status::BadRequest, Some(Json(login_response)))
                }
            },
            None => {
                login_response
                    .errors
                    .push("Username does not exist".to_string());
                (Status::BadRequest, Some(Json(login_response)))
            },
        },
        _ => (Status::InternalServerError, None),
    }
}
