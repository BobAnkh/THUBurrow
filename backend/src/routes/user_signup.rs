// use rocket::http::{Cookie, CookieJar};
// use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
// use rocket::serde::{Serialize};
use rocket::http::Status;

use sea_orm::QueryFilter;
use sea_orm::{entity::*, ActiveModelTrait};
// use uuid::Uuid;

use crate::db;
use crate::db::new_user::Entity as User;
use crate::pool::PgDb;
use crate::req::user::*;

// use chrono::Local;
// use crypto::digest::Digest;
// use crypto::sha3::Sha3;

use idgenerator::IdHelper;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/users", routes![user_sign_up])
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
    if user.email.ends_with("tsinghua.edu.cn") == false {
        signup_response
            .errors
            .push("Illegal Email Address".to_string());
    }
    // check if email address is duplicated, add corresponding error if so
    let email_dup_result = User::find()
        .filter(db::new_user::Column::Email.eq(user.email))
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
        .filter(db::new_user::Column::Username.eq(user.username))
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
        return (Status::BadRequest, Json(signup_response));
    } else {
        signup_response.success = true;
    }
    // add salt to password
    /*TODO */
    // generate uid
    let uid: i64 = IdHelper::next_id();
    // fill the row
    let new_user = db::new_user::ActiveModel {
        uid: Set(uid.to_owned()),
        username: Set(Some(user.username.to_string()).to_owned()),
        password: Set(Some(user.password.to_string()).to_owned()),
        email: Set(Some(user.email.to_string()).to_owned()),
        ..Default::default()
    };
    // insert the row in database
    let res = new_user.insert(&db).await.expect("Cannot save user");
    println!("{:?}", res.uid);
    // return the response
    (Status::Accepted, Json(signup_response))
}
