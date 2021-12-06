use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::Request;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use sea_orm::{entity::*, ActiveModelTrait};
use uuid::Uuid;

use crate::db;
use crate::pool::{PgDb, PulsarSearchProducerMq, RedisDb, RocketPulsarProducer};
use crate::req::pulsar::*;
use crate::req::user::*;
use crate::utils::auth::{self, Auth, AuthTokenError, ValidToken};

use chrono::prelude::*;
use chrono::Local;
use crypto::digest::Digest;
use crypto::sha3::Sha3;

use idgenerator::IdHelper;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .mount(
            "/sample",
            routes![
                hello,
                hi,
                redirect_user_by_id,
                user_login,
                user_sign_up,
                redis_save,
                redis_read,
                auth_name,
                auth_new,
                sso_test,
                pulsar_produce
            ],
        )
        .register(
            "/sample/auth/new",
            catchers![auth_new_bad_request, auth_new_unauthorized],
        )
}

#[get("/test/sso")]
pub async fn sso_test(a: auth::Auth) -> Json<i64> {
    Json(a.id)
}

#[get("/auth/<name>")]
async fn auth_name(auth: Result<Auth, AuthTokenError>, name: &str) -> String {
    if let Err(e) = auth {
        match e {
            AuthTokenError::Invalid => return "Invalid token".to_string(),
            AuthTokenError::Missing => return "Missing token".to_string(),
            AuthTokenError::DatabaseErr => return "DatabaseErr token".to_string(),
        }
    }
    format!("Hello, {}!", name)
}

#[get("/auth/new/<name>")]
async fn auth_new(auth: Auth, name: &str) -> String {
    format!("Hello, {}, your id is {}!", name, auth.id)
}

#[catch(400)]
async fn auth_new_bad_request(request: &Request<'_>) -> String {
    let user_result = request
        .local_cache_async(async { auth::auth_token(request).await })
        .await;
    match user_result {
        Some(e) => match e {
            ValidToken::Invalid => "Invalid token".to_string(),
            ValidToken::Missing => "Missing token".to_string(),
            ValidToken::DatabaseErr => "DatabaseErr token".to_string(),
            ValidToken::Valid(id) => format!("User Id found: {}", id),
            ValidToken::Refresh(id) => format!("User Id found: {}", id),
        },
        None => "Valid token".to_string(),
    }
}

#[catch(401)]
async fn auth_new_unauthorized(request: &Request<'_>) -> String {
    let user_result = request
        .local_cache_async(async { auth::auth_token(request).await })
        .await;
    match user_result {
        Some(e) => match e {
            ValidToken::Invalid => "Invalid token".to_string(),
            ValidToken::Missing => "Missing token".to_string(),
            ValidToken::DatabaseErr => "DatabaseErr token".to_string(),
            ValidToken::Valid(id) => format!("User Id found: {}", id),
            ValidToken::Refresh(id) => format!("User Id found: {}", id),
        },
        None => "Valid token".to_string(),
    }
}

#[get("/pulsar/<burrow_id>")]
async fn pulsar_produce(pulsar: Connection<PulsarSearchProducerMq>, burrow_id: i64) -> String {
    let mut producer = match pulsar
        .get_producer("persistent://public/default/search")
        .await
    {
        Ok(producer) => producer,
        Err(e) => {
            println!("{:?}", e);
            return "Error".to_string();
        }
    };
    let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
    let data = PulsarSearchBurrowData {
        burrow_id: burrow_id,
        title: format!("This is burrow NO.{}", burrow_id),
        introduction: format!("Content of burrow.{}", burrow_id),
        update_time: now,
    };
    let msg = PulsarSearchData::CreateBurrow(data);
    match producer.send(msg).await {
        // Ok(r) => match r.await {
        //     Ok(cs) => format!("send data successfully!, {}", cs.producer_id),
        //     Err(e) => format!("Err: {}", e),
        // },
        // Err(e) => format!("Err: {}", e),
        Ok(_) => format!("send data to pulsar successfully!.NO.{}", burrow_id),
        Err(e) => format!("Err: {}", e),
    }
    // let f1 = r.await?;
}

#[get("/hello/<name>", rank = 2)]
async fn hello(name: &str) -> String {
    let new_id: i64 = IdHelper::next_id();
    format!("Hello, {}! This is your id: {}", name, new_id)
}

#[get("/hello/<id>", rank = 1)]
async fn hi(id: i32) -> String {
    format!("Sending a number, {}!", id)
}

#[get("/redirect/<id>")]
async fn redirect_user_by_id(id: i32) -> String {
    hi(id).await
}

#[get("/redis/<name>")]
async fn redis_save(
    db: Connection<RedisDb>,
    name: &str,
) -> Result<String, status::NotFound<String>> {
    let redis_result: Result<String, redis::RedisError> = redis::cmd("SET")
        .arg(&name)
        .arg(123456789)
        .query_async(db.into_inner().as_mut())
        .await;
    match redis_result {
        Ok(s) => Ok(format!("{}, {}", name, s)),
        _ => Err(status::NotFound("Redis cannot save".to_string())),
    }
}

#[get("/redis/retrieve/<name>")]
async fn redis_read(
    db: Connection<RedisDb>,
    name: &str,
) -> Result<String, status::NotFound<String>> {
    let redis_result: Result<String, redis::RedisError> = redis::cmd("GET")
        .arg(name)
        .query_async(db.into_inner().as_mut())
        .await;
    match redis_result {
        Ok(s) => Ok(format!("{}, {}", name, s)),
        _ => Err(status::NotFound("Redis cannot read".to_string())),
    }
}

#[get("/login/<uuid>")]
async fn user_login(
    cookies: &CookieJar<'_>,
    db: Connection<PgDb>,
    uuid: Uuid,
) -> Result<Json<UserData>, status::NotFound<String>> {
    match cookies.get_private("token") {
        Some(cookie) => {
            let token = cookie.value().to_string();
            println!("{:?}", uuid);
            match db::user::Entity::find_by_id(uuid)
                .one(&db.into_inner())
                .await
            {
                Ok(Some(user)) => match user.token {
                    Some(s) => {
                        if s != token {
                            Err(status::NotFound("Wrong token".to_string()))
                        } else {
                            Ok(Json(UserData {
                                id: user.uuid,
                                name: user.username.unwrap(),
                            }))
                        }
                    }
                    _ => Err(status::NotFound("No token found".to_string())),
                },
                _ => Err(status::NotFound("Can not find this user".to_string())),
            }
        }
        _ => Err(status::NotFound("No cookie".to_string())),
    }
}

#[post("/sign-up", data = "<user_info>", format = "json")]
async fn user_sign_up(
    db: Connection<PgDb>,
    cookies: &CookieJar<'_>,
    user_info: Json<UserLoginInfo<'_>>,
) -> Json<Uuid> {
    // get user info from request
    let user = user_info.into_inner();
    // generate user token from user info
    let user_key: String =
        Local::now().timestamp_millis().to_string() + user.username + user.password;
    let mut hash_sha3 = Sha3::sha3_256();
    hash_sha3.input_str(&user_key);
    let token = hash_sha3.result_str();
    println!("{}", user_key);
    // generate uuid
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, user_key.as_bytes());
    // build cookie
    let cookie = Cookie::build("token", token.clone())
        .domain(".thuburrow.com")
        .path("/")
        .same_site(SameSite::Strict)
        .secure(true)
        .http_only(true)
        .max_age(time::Duration::weeks(1))
        .finish();
    // set cookie
    cookies.add_private(cookie);
    // fill the row
    let user = db::user::ActiveModel {
        uuid: Set(uuid.to_owned()),
        username: Set(Some(user.username.to_string())),
        password: Set(Some(user.password.to_string())),
        token: Set(Some(token)),
        ..Default::default()
    };
    // insert the row in database
    let res = user
        .insert(&db.into_inner())
        .await
        .expect("Cannot save user");
    println!("{}", res.token.unwrap().unwrap());
    // return the response
    Json(res.uuid.unwrap())
}
