// use rocket::http::{Cookie, CookieJar};
// use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
// use rocket::serde::{Serialize};
use rocket::http::Status;

use sea_orm::entity::*;
use sea_orm::QueryFilter;
// use uuid::Uuid;

use crate::db;
use crate::db::new_user::Entity as User;
use crate::pool::PgDb;
use crate::req::user::*;

// use chrono::Local;
// use crypto::digest::Digest;
// use crypto::sha3::Sha3;

// use idgenerator::IdHelper;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/users", routes![user_log_in])
}

#[post("/login", data = "<user_info>", format = "json")]
pub async fn user_log_in(
    db: Connection<PgDb>,
    user_info: Json<UserLoginInfo<'_>>,
) -> (Status, Json<UserLoginResponse>) {
    // create a response struct
    let mut login_response = UserLoginResponse {
        success: false,
        errors: Vec::new(),
    };
    // get user info from request
    let user = user_info.into_inner();
    // check if username is existed, add corresponding error if so
    let username_existence_result: Option<db::new_user::Model> = User::find()
        .filter(db::new_user::Column::Username.eq(user.username))
        .one(&db)
        .await
        .expect("cannot fetch username data from pgdb");
    match username_existence_result {
        Some(matched_user) => {
            if matched_user.password.eq(&Some(user.password.to_string())) {
                login_response.success = true;
                (Status::Accepted, Json(login_response))
            } else {
                login_response.errors.push("Wrong password".to_string());
                (Status::BadRequest, Json(login_response))
            }
        }
        None => {
            login_response
                .errors
                .push("Username does not exist".to_string());
            (Status::BadRequest, Json(login_response))
        }
    }
}
