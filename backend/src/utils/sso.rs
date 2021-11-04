use crate::pool::RedisDb;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::{Cookie, SameSite, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::State;

pub struct SsoAuth {
    pub id: i64,
}

#[derive(Debug)]
pub enum AuthTokenError {
    Missing,
    Invalid,
    DatabaseErr,
}

pub enum ValidToken {
    Valid(i64),
    Refresh(i64),
    Invalid,
    DatabaseErr,
    Missing,
}

async fn is_valid<'r>(
    request: &'r Request<'_>,
    token: &str,
    con: &mut redis::aio::Connection,
) -> ValidToken {
    let redis_result: Result<u32, redis::RedisError> = redis::cmd("EXPIRE")
        .arg(token)
        // .arg(4*3600)
        .arg(20)
        .query_async(con)
        .await;
    match redis_result {
        // token exists
        Ok(1) => {
            let get_result: Result<i64, redis::RedisError> =
                redis::cmd("GET").arg(token).query_async(con).await;
            match get_result {
                Ok(id) => ValidToken::Valid(id),
                _ => ValidToken::DatabaseErr,
            }
        }
        // token does not exist
        Ok(_) => {
            // hash token to refresh token
            let mut hash_sha3 = Sha3::sha3_384();
            hash_sha3.input_str(&token);
            let refresh_token = hash_sha3.result_str();
            // generate new token
            let new_token: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect();
            // hash new token to new refresh token
            let mut hash_sha3 = Sha3::sha3_384();
            hash_sha3.input_str(&new_token);
            let new_refresh_token = hash_sha3.result_str();
            // try to rename old refresh token to new refresh token
            let refresh_result: Result<u32, redis::RedisError> = redis::cmd("RENAMENX")
                .arg(&refresh_token)
                .arg(&new_refresh_token)
                .query_async(con)
                .await;
            match refresh_result {
                // successfully rename refresh token to new refresh token
                Ok(1) => {
                    // find id by get refresh_token 
                    let get_result: Result<i64, redis::RedisError> =
                        redis::cmd("GET").arg(&new_refresh_token).query_async(con).await;
                    let id: i64 = match get_result {
                        Ok(id) => {
                            // set id -> new_token
                            let old_token_get: Result<String, redis::RedisError> = 
                                redis::cmd("GETSET").arg(id).arg(&new_token).query_async(con).await;
                            // clear old_token -> id
                            match old_token_get {
                                Ok(old_token) => {
                                    let _: Result<i64, redis::RedisError> = 
                                        redis::cmd("DEL").arg(&old_token).query_async(con).await;
                                },
                                _ => return ValidToken::DatabaseErr,
                            };
                            // set new_token -> id
                            let refresh_set: Result<String, redis::RedisError> = redis::cmd("SETEX")
                                .arg(&new_token)
                                // .arg(4*3600)
                                .arg(20)
                                .arg(id)
                                .query_async(con)
                                .await;
                            match refresh_set {
                                Ok(_) => {
                                    // set cookie to the new token
                                    let cookie = Cookie::build("token", new_token)
                                        .domain("thuburrow.com")
                                        .path("/")
                                        .same_site(SameSite::None)
                                        .finish();
                                    request.cookies().add_private(cookie);
                                },
                                _ => return ValidToken::DatabaseErr,
                            };
                            id
                        },
                        _ => return ValidToken::DatabaseErr,
                        
                    };
                    ValidToken::Refresh(id)
                },
                // database error, new refresh token already exists
                Ok(_) => ValidToken::DatabaseErr,
                // refresh token does not exist
                _ => {
                    // println!("{:?}", e.to_string());
                    ValidToken::Invalid
                },
            }
                    
        },
        // database connection error
        _ => ValidToken::DatabaseErr,
    }
}

pub async fn auth_token<'r>(request: &'r Request<'_>) -> Option<ValidToken> {
    let db: &State<RedisDb>;
    match request.guard::<&State<RedisDb>>().await.succeeded() {
        None => {
            return Some(ValidToken::DatabaseErr);
        }
        Some(d) => {
            db = d;
        }
    }
    let mut redis_manager = RedisDb::get_redis_con(db).await;
    match request.cookies().get_private("token") {
        // no token in cookie
        None => Some(ValidToken::Missing),
        // get token from cookie and valid it
        Some(token) => match is_valid(request, token.value(), redis_manager.as_mut()).await {
            ValidToken::Valid(id) => Some(ValidToken::Valid(id)),
            ValidToken::Refresh(id) => Some(ValidToken::Refresh(id)),
            ValidToken::DatabaseErr => Some(ValidToken::DatabaseErr),
            _ => Some(ValidToken::Invalid),
        },
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SsoAuth {
    type Error = AuthTokenError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_result = request
            .local_cache_async(async { auth_token(request).await })
            .await;
        match auth_result {
            Some(msg) => match msg {
                ValidToken::Missing => {
                    Outcome::Failure((Status::BadRequest, AuthTokenError::Missing))
                }
                ValidToken::Invalid => {
                    Outcome::Failure((Status::Unauthorized, AuthTokenError::Invalid))
                }
                ValidToken::DatabaseErr => {
                    Outcome::Failure((Status::InternalServerError, AuthTokenError::DatabaseErr))
                }
                ValidToken::Refresh(id) => Outcome::Success(SsoAuth { id: id.clone() }),
                ValidToken::Valid(id) => Outcome::Success(SsoAuth { id: id.clone() }),
            },
            None => Outcome::Failure((Status::InternalServerError, AuthTokenError::DatabaseErr)),
        }
    }
}
