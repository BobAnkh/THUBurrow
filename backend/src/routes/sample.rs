use std::time;

use pulsar::OperationRetryOptions;
use rocket::http::{Cookie, CookieJar};
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use sea_orm::{entity::*, ActiveModelTrait};
use uuid::Uuid;

use crate::db;
use crate::pool::{PgDb, PulsarSearchProducerMq, RedisDb};
use crate::req::{user::*, sample_req::*};

use chrono::Local;
use crypto::digest::Digest;
use crypto::sha3::Sha3;

use idgenerator::IdHelper;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/sample",
        routes![
            hello,
            hi,
            redirect_user_by_id,
            user_login,
            user_sign_up,
            redis_save,
            redis_read,
            pulsar_produce
        ],
    )
}

#[get("/pulsar/<name>")]
async fn pulsar_produce(mut producer: Connection<PulsarSearchProducerMq>, name: &str) -> String {
    let operation = r#"{
        "operation_level": "burrow",
        "operation_type": "new",
        "index": 1,
        "operation_time":2342341515,
        "data": "Hello motherfucker!"
    }"#;
    let msg:TestData = serde_json::from_str(operation).unwrap();
    match producer.send(msg).await {
        // Ok(r) => match r.await {
        //     Ok(cs) => format!("send data successfully!, {}", cs.producer_id),
        //     Err(e) => format!("Err: {}", e),
        // },
        // Err(e) => format!("Err: {}", e),
        Ok(_) => format!("send data to pulsarsuccessfully!,{}",name),
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
        .arg(&[name, "bar"])
        .query_async(&mut *db.into_inner())
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
        .query_async(&mut *db.into_inner())
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
            match db::user::Entity::find_by_id(uuid).one(&db).await {
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
    user_info: Json<UserInfo<'_>>,
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
        .domain("thuburrow.com")
        .path("/")
        .finish();
    // set cookie
    cookies.add_private(cookie);
    // fill the row
    let user = db::user::ActiveModel {
        uuid: Set(uuid.to_owned()),
        username: Set(Some(user.username.to_string()).to_owned()),
        password: Set(Some(user.password.to_string()).to_owned()),
        token: Set(Some(token).to_owned()),
        ..Default::default()
    };
    // insert the row in database
    let res = user.insert(&db).await.expect("Cannot save user");
    println!("{}", res.token.unwrap().unwrap());
    // return the response
    Json(res.uuid.unwrap())
}
