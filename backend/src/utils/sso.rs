use crate::pool::RedisDb;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::{Cookie, SameSite, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::State;

pub struct SsoAuth;

#[derive(Debug)]
pub enum AuthTokenError {
    Missing,
    Invalid,
    DatabaseErr,
}

pub enum ValidToken {
    Valid,
    Refresh(String),
    Invalid,
    DatabaseErr,
}

async fn is_valid(token: &str, con: &mut redis::aio::Connection) -> ValidToken {
    let redis_result: Result<u32, redis::RedisError> = redis::cmd("EXPIRE")
        .arg(token)
        .arg(14400)
        .query_async(con)
        .await;
    match redis_result {
        // token exists
        Ok(1) => ValidToken::Valid,
        // token does not exist
        Ok(_) => {
            // hash token to refresh token
            let mut hash_sha3 = Sha3::sha3_256();
            hash_sha3.input_str(&token);
            let refresh_token = hash_sha3.result_str();
            // generate new token
            // TODO: might be replace
            let new_token: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();
            // hash new token to new refresh token
            hash_sha3.input_str(&new_token);
            let new_refresh_token = hash_sha3.result_str();
            // try to rename old refresh token to new refresh token
            let refresh_result: Result<u32, redis::RedisError> = redis::cmd("RENAMENX")
                .arg(refresh_token)
                .arg(new_refresh_token)
                .query_async(con)
                .await;
            match refresh_result {
                // successfully rename refresh token to new refresh token
                Ok(1) => {
                    // set token
                    let refresh_set: Result<String, redis::RedisError> = redis::cmd("SETEX")
                        .arg(&new_token)
                        .arg(14400)
                        .arg("user")
                        .query_async(con)
                        .await;
                    match refresh_set {
                        Ok(_) => ValidToken::Refresh(new_token),
                        _ => ValidToken::DatabaseErr,
                    }
                }
                // database error, new refresh token already exists
                Ok(_) => ValidToken::DatabaseErr,
                // refresh token does not exist
                _ => ValidToken::Invalid,
            }
        }
        // database connection error
        _ => ValidToken::DatabaseErr,
    }
}

pub async fn auth_token<'r>(request: &'r Request<'_>) -> Option<AuthTokenError> {
    let db: &State<RedisDb>;
    match request.guard::<&State<RedisDb>>().await.succeeded() {
        None => {
            return Some(AuthTokenError::DatabaseErr);
        }
        Some(d) => {
            db = d;
        }
    }
    let mut redis_manager = RedisDb::get_redis_con(db).await;
    match request.cookies().get_private("token") {
        // no token in cookie
        None => Some(AuthTokenError::Missing),
        // get token from cookie and valid it
        Some(token) => match is_valid(token.value(), redis_manager.as_mut()).await {
            ValidToken::Valid => None,
            ValidToken::Refresh(new_token) => {
                let cookie = Cookie::build("token", new_token.clone())
                    .domain("thuburrow.com")
                    .path("/")
                    .same_site(SameSite::None)
                    .finish();
                request.cookies().add_private(cookie);
                None
            }
            ValidToken::Invalid => Some(AuthTokenError::Invalid),
            ValidToken::DatabaseErr => Some(AuthTokenError::DatabaseErr),
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
            Some(err) => match err {
                AuthTokenError::Missing => {
                    Outcome::Failure((Status::BadRequest, AuthTokenError::Missing))
                }
                AuthTokenError::Invalid => {
                    Outcome::Failure((Status::Unauthorized, AuthTokenError::Invalid))
                }
                AuthTokenError::DatabaseErr => {
                    Outcome::Failure((Status::InternalServerError, AuthTokenError::DatabaseErr))
                }
            },
            None => Outcome::Success(SsoAuth),
        }
    }
}
