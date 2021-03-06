//! Module for authentication

use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::{private::cookie::CookieBuilder, Cookie, SameSite, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::State;

use crate::config::user::*;
use crate::models::error::*;
use crate::pool::RedisDb;

/// Usage of Auth
///
/// ## Example
///
/// ```ignore
/// use rocket::Request;
/// use rocket::{Build, Rocket};
/// use crate::models::error::*;
/// use crate::utils::auth::{self, Auth, ValidToken};
/// pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
///     rocket
///         .mount(
///             "/sample",
///             routes![
///                 auth_name,
///                 auth_new,
///             ],
///         )
///         .register(
///             "/sample/auth/new",
///             catchers![auth_new_bad_request, auth_new_unauthorized],
///         )
/// }
///
/// #[get("/auth/<name>")]
/// async fn auth_name(auth: Result<Auth, ErrorResponse>, name: &str) -> String {
///     if let Err(e) = auth {
///         return format!("{:?}", e);
///     }
///     format!("Hello, {}!", name)
/// }
///
/// #[get("/auth/new/<name>")]
/// async fn auth_new(auth: Auth, name: &str) -> String {
///     format!("Hello, {}, your id is {}!", name, auth.id)
/// }
///
/// #[catch(400)]
/// async fn auth_new_bad_request(request: &Request<'_>) -> String {
///     let user_result = request
///         .local_cache_async(async { auth::auth_token(request).await })
///         .await;
///     match user_result {
///         Some(e) => match e {
///             ValidToken::Invalid => "Invalid token".to_string(),
///             ValidToken::Missing => "Missing token".to_string(),
///             ValidToken::DatabaseErr => "DatabaseErr token".to_string(),
///             ValidToken::Valid(id) => format!("User Id found: {}", id),
///             ValidToken::Refresh(id) => format!("User Id found: {}", id),
///         },
///         None => "Valid token".to_string(),
///     }
/// }
///
/// #[catch(401)]
/// async fn auth_new_unauthorized(request: &Request<'_>) -> String {
///     let user_result = request
///         .local_cache_async(async { auth::auth_token(request).await })
///         .await;
///     match user_result {
///         Some(e) => match e {
///             ValidToken::Invalid => "Invalid token".to_string(),
///             ValidToken::Missing => "Missing token".to_string(),
///             ValidToken::DatabaseErr => "DatabaseErr token".to_string(),
///             ValidToken::Valid(id) => format!("User Id found: {}", id),
///             ValidToken::Refresh(id) => format!("User Id found: {}", id),
///         },
///         None => "Valid token".to_string(),
///     }
/// }
/// ```
pub struct Auth {
    pub id: i64,
}

/// Message of token check
///
/// ## Parameter
///
/// - `ValidToken::Invalid`: Invalid token
/// - `ValidToken::Missing`: Missing token
/// - `ValidToken::DatabaseErr`: Database error
/// - `ValidToken::Valid(id)`: Valid token, will provide uid
/// - `ValidToken::Refresh(id)`: Refresh token, will provide uid
pub enum ValidToken {
    Valid(i64),
    Refresh(i64),
    Invalid,
    DatabaseErr,
    Missing,
}

pub async fn set_token(
    uid: i64,
    kv_conn: &mut redis::aio::Connection,
) -> Result<String, ErrorResponse> {
    // generate token and refresh token
    let token: String = std::iter::repeat(())
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
        .arg(TOKEN_TO_ID_EX)
        .arg(uid)
        .query_async(kv_conn)
        .await;
    match uid_result {
        Ok(s) => info!("[LOGIN] setex token->id: {:?} -> {}", &token, s),
        Err(e) => {
            error!(
                "[LOGIN] failed to set token -> id when login. RedisError: {:?}",
                e
            );
            return Err(ErrorResponse::default());
        }
    };
    // set refresh_token -> id
    let uid_result: Result<String, redis::RedisError> = redis::cmd("SETEX")
        .arg(&refresh_token)
        .arg(REF_TOKEN_TO_ID_EX)
        .arg(uid)
        .query_async(kv_conn)
        .await;
    match uid_result {
        Ok(s) => info!(
            "[LOGIN] setex refresh_token->id: {:?} -> {}",
            &refresh_token, s
        ),
        Err(e) => {
            error!(
                "[LOGIN] failed to set refresh_token -> id when login. RedisError: {:?}",
                e
            );
            return Err(ErrorResponse::default());
        }
    };
    // get old token and set new token by getset id -> token
    let _ = delete_token(uid, kv_conn).await?;
    let new_token_set: Result<String, redis::RedisError> = redis::cmd("SETEX")
        .arg(uid)
        .arg(ID_TO_TOKEN_EX)
        .arg(&token)
        .query_async(kv_conn)
        .await;
    match new_token_set {
        Ok(_) => {
            info!("[LOGIN] set id->token: {} -> {:?}", uid, token);
            Ok(token)
        }
        Err(e) => {
            error!(
                "[LOGIN] failed to set id -> token when login. RedisError: {:?}",
                e
            );
            Err(ErrorResponse::default())
        }
    }
}

pub async fn delete_token(
    uid: i64,
    kv_conn: &mut redis::aio::Connection,
) -> Result<String, ErrorResponse> {
    let old_token_get: Result<Option<String>, redis::RedisError> =
        redis::cmd("GET").arg(uid).query_async(kv_conn).await;
    match old_token_get {
        Ok(res) => match res {
            // if old token -> id exists
            Some(old_token) => {
                info!("[TOKEN] find old token: {}, continue...", old_token);
                // clear old token -> id
                let delete_result: Result<i64, redis::RedisError> =
                    redis::cmd("DEL").arg(&old_token).query_async(kv_conn).await;
                match delete_result {
                    Ok(1) => info!("[TOKEN] delete token->id"),
                    Ok(0) => info!("[TOKEN] no token->id found"),
                    Ok(_) => {
                        error!("[TOKEN] failed to delete refresh_token -> id when login.");
                        return Err(ErrorResponse::default());
                    }
                    Err(e) => {
                        error!(
                            "[TOKEN] failed to delete token -> id when login. RedisError: {:?}",
                            e
                        );
                        return Err(ErrorResponse::default());
                    }
                };
                // find old refresh_token by hashing old token
                let mut hash_sha3 = Sha3::sha3_384();
                hash_sha3.input_str(&old_token);
                let old_refresh_token = hash_sha3.result_str();
                // clear old refresh_token -> id
                let delete_result: Result<i64, redis::RedisError> = redis::cmd("DEL")
                    .arg(&old_refresh_token)
                    .query_async(kv_conn)
                    .await;
                match delete_result {
                    Ok(1) => info!("[TOKEN] delete ref_token->id"),
                    Ok(0) => info!("[TOKEN] no ref_token->id found"),
                    Ok(_) => {
                        error!("[TOKEN] failed to delete refresh_token -> id when login.");
                        return Err(ErrorResponse::default());
                    }
                    Err(e) => {
                        error!("[TOKEN] failed to delete refresh_token -> id when login. RedisError: {:?}", e);
                        return Err(ErrorResponse::default());
                    }
                };
                Ok("Success".to_string())
            }
            None => {
                info!("[LOGIN] no id -> token found");
                Ok("Not-Found".to_string())
            }
        },
        Err(e) => {
            error!(
                "[LOGIN] failed to get id -> token when login. RedisError: {:?}",
                e
            );
            Err(ErrorResponse::default())
        }
    }
}

async fn is_valid<'r>(
    request: &'r Request<'_>,
    token: &str,
    con: &mut redis::aio::Connection,
) -> ValidToken {
    let redis_result: Result<u32, redis::RedisError> = redis::cmd("EXPIRE")
        .arg(token)
        .arg(4 * 3600)
        .query_async(con)
        .await;
    match redis_result {
        // token exists
        Ok(1) => {
            let get_result: Result<i64, redis::RedisError> =
                redis::cmd("GET").arg(token).query_async(con).await;
            match get_result {
                Ok(id) => {
                    info!("[SSO] token -> id exists");
                    ValidToken::Valid(id)
                }
                _ => ValidToken::DatabaseErr,
            }
        }
        // token does not exist
        Ok(_) => {
            info!("[SSO] token -> id has expired, try to find refresh_token...");
            // hash token to refresh token
            let mut hash_sha3 = Sha3::sha3_384();
            hash_sha3.input_str(token);
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
                    info!("[SSO] refresh_token exists.");
                    // find id by get refresh_token
                    let get_result: Result<i64, redis::RedisError> = redis::cmd("GET")
                        .arg(&new_refresh_token)
                        .query_async(con)
                        .await;
                    let id: i64 = match get_result {
                        Ok(id) => {
                            // set id -> new_token
                            let old_token_get: Result<String, redis::RedisError> =
                                redis::cmd("GETSET")
                                    .arg(id)
                                    .arg(&new_token)
                                    .query_async(con)
                                    .await;
                            // clear old_token -> id
                            match old_token_get {
                                Ok(old_token) => {
                                    info!("[SSO] set id -> new_token");
                                    let _: Result<i64, redis::RedisError> =
                                        redis::cmd("DEL").arg(&old_token).query_async(con).await;
                                    info!("[SSO] delete old_token -> id");
                                }
                                _ => return ValidToken::DatabaseErr,
                            };
                            // set new_token -> id
                            let refresh_set: Result<String, redis::RedisError> =
                                redis::cmd("SETEX")
                                    .arg(&new_token)
                                    .arg(4 * 3600i32)
                                    .arg(id)
                                    .query_async(con)
                                    .await;
                            match refresh_set {
                                Ok(_) => {
                                    // set cookie to the new token
                                    let cookie =
                                        Cookie::build("token", new_token).cookie_options().finish();
                                    request.cookies().add_private(cookie);
                                    info!("[SSO] set new_token -> id");
                                }
                                _ => return ValidToken::DatabaseErr,
                            };
                            id
                        }
                        _ => return ValidToken::DatabaseErr,
                    };
                    ValidToken::Refresh(id)
                }
                // database error, new refresh token already exists
                Ok(0) => {
                    error!("[SSO] new_refresh_token already exists in redis.");
                    ValidToken::DatabaseErr
                }
                // refresh token does not exist
                _ => {
                    info!("[SSO] refresh_token expired, need to re-login.");
                    ValidToken::Invalid
                }
            }
        }
        // database connection error
        _ => {
            error!("[SSO] failed to connect redis.");
            ValidToken::DatabaseErr
        }
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
impl<'r> FromRequest<'r> for Auth {
    type Error = ErrorResponse;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_result = request
            .local_cache_async(async { auth_token(request).await })
            .await;
        match auth_result {
            Some(msg) => match msg {
                ValidToken::Missing => Outcome::Failure((
                    Status::Unauthorized,
                    ErrorResponse::build(
                        ErrorCode::AuthTokenMissing,
                        "Authentication token is missing.",
                    ),
                )),
                ValidToken::Invalid => Outcome::Failure((
                    Status::Unauthorized,
                    ErrorResponse::build(
                        ErrorCode::AuthTokenInvalid,
                        "Authentication token is invalid.",
                    ),
                )),
                ValidToken::DatabaseErr => {
                    Outcome::Failure((Status::InternalServerError, ErrorResponse::default()))
                }
                ValidToken::Refresh(id) => Outcome::Success(Auth { id: *id }),
                ValidToken::Valid(id) => Outcome::Success(Auth { id: *id }),
            },
            None => Outcome::Failure((Status::InternalServerError, ErrorResponse::default())),
        }
    }
}

pub trait CookieOptions {
    fn cookie_options(self) -> Self;
}

impl CookieOptions for CookieBuilder<'_> {
    fn cookie_options(self) -> Self {
        self.domain(".thuburrow.com")
            .path("/")
            .same_site(SameSite::Strict)
            .secure(true)
            .http_only(true)
            .max_age(time::Duration::weeks(1))
    }
}
