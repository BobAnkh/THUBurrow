// use rocket::http::{Cookie, CookieJar};
// use rocket::response::status;
use rocket::serde::json::Json;
// use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use rocket::serde::{Serialize};

use sea_orm::{entity::*, ActiveModelTrait};
use sea_orm::QueryFilter;
use uuid::Uuid;

use crate::pool::Db;
use crate::db;
use crate::req::user::*;
use crate::db::user::Entity as User;

// use chrono::Local;
use crypto::digest::Digest;
use crypto::sha3::Sha3;

#[derive(Serialize)]
pub struct UserSignupResponse {
    success: bool,
    error: Vec<String>,
}

#[post("/sign-up", data = "<user_info>", format = "json")]
pub async fn user_sign_up(db: Connection<Db>, user_info: Json<UserInfo<'_>>) -> Json<UserSignupResponse> {
    // create a response struct
    let mut signup_response = UserSignupResponse{
        success: false,
        error: vec![],
    };
    // get user info from request
    let user = user_info.into_inner();
    // check if email address is valid, add corresponding error if so
    if user.email.contains("@tsinghua.edu.cn") == false {
        signup_response.error.push("email_illegal".to_string());
    }
    // check if email address is duplicated, add corresponding error if so
    let email_dup_result = User::find()
        .filter(db::user::Column::Email.eq(user.email))
        .one(&db)
        .await
        .expect("failed to find by email in db");
    if let Some(_s) = email_dup_result {
        signup_response.error.push("email_dup".to_string());
    }
    // check if username is duplicated, add corresponding error if so
    let username_dup_result =  User::find()
        .filter(db::user::Column::Username.eq(user.username))
        .one(&db)
        .await
        .expect("failed to fing by username in db");    
    if let Some(_s) = username_dup_result {
        signup_response.error.push("username_dup".to_string());
    }
    // if error exists, refuse to add user
    match signup_response.error.is_empty() {
        false => {
            return Json(signup_response);
        }
        true => {
            signup_response.success = true;
        }
    }
    // add salt to password
        /*TODO */
    // generate uuid from user info
    let user_key: String = String::from(user.username) + user.password;
    let mut hash_sha3 = Sha3::sha3_256();
    hash_sha3.input_str(&user_key);
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, user_key.as_bytes());
    println!("{:?}", uuid);
    // fill the row
    let user = db::user::ActiveModel {
        uuid: Set(uuid.to_owned()),
        username: Set(Some(user.username.to_string()).to_owned()),
        password: Set(Some(user.password.to_string()).to_owned()),
        email: Set(Some(user.email.to_string()).to_owned()),
        ..Default::default()
    };
    // insert the row in database
    let res = user
        .insert(&db)
        .await
        .expect("Cannot save user");
    println!("{:?}", res.uuid);
    // return the response
    Json(signup_response)
}
