//! Routes for storage

use chrono::{FixedOffset, Utc};
use crypto::digest::Digest;
use crypto::md5::Md5;
use rocket::http::{ContentType, Status};
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::{entity::*, ActiveModelTrait};

use crate::config::storage::*;
use crate::db::{self, image, prelude::*};
use crate::models::error::*;
use crate::models::storage::{ReferrerCheck, SaveImage};
use crate::pool::{MinioImageStorage, PgDb};
use crate::utils::auth::Auth;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/storage",
        routes![upload_image, download_image, get_images],
    )
}

/// Upload image
///
/// ## Parameters
///
/// - `Auth`: Authenticated User
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<MinioImageStorage>`: Image storage connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `String`: Filename of image
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///     - `ErrorCode::DatabaseErr`
///
#[post("/images", data = "<image>")]
async fn upload_image(
    auth: Auth,
    db: Connection<PgDb>,
    bucket: Connection<MinioImageStorage>,
    image: SaveImage,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    info!("[IMAGE] User {} id uploading image.", auth.id);
    let pg_con = db.into_inner();
    // put a file
    let mut hash_md5 = Md5::new();
    hash_md5.input(image.content.as_slice());
    let filename = hash_md5.result_str() + "." + image.content_type.to_string().as_str();
    let image_size = image.content.len() as i32;
    match UserStatus::find_by_id(auth.id).one(&pg_con).await {
        Ok(opt_state) => match opt_state {
            Some(state) => {
                if state.file_num >= *MAX_IMAGE_NUM {
                    return (
                        Status::TooManyRequests,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::RateLimit,
                            "Store too many images.",
                        ))),
                    );
                }
                match bucket
                    .put_object(filename.as_str(), image.content.as_slice())
                    .await
                {
                    Ok((_, 200)) => {
                        let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
                        let record = image::ActiveModel {
                            filename: Set(filename.to_owned()),
                            uid: Set(auth.id),
                            size: Set(image_size),
                            create_time: Set(now.to_owned()),
                            last_download_time: Set(now),
                            ..Default::default()
                        };
                        let file_num = state.file_num;
                        let file_capacity = state.file_capacity;
                        let mut ust: db::user_status::ActiveModel = state.into();
                        ust.file_num = Set(file_num + 1);
                        ust.file_capacity = Set(file_capacity + image_size as i64);
                        let _ = record.insert(&pg_con).await;
                        let _ = ust.update(&pg_con).await;
                        (Status::Ok, Ok(filename))
                    }
                    Ok((_, code)) => (
                        Status::InternalServerError,
                        Err(Json(ErrorResponse::build(
                            ErrorCode::Unknown,
                            format!("code: {}", code),
                        ))),
                    ),
                    Err(e) => {
                        log::error!("[Image-Storage] Database Error {:?}", e);
                        (
                            Status::InternalServerError,
                            Err(Json(ErrorResponse::default())),
                        )
                    }
                }
            }
            None => {
                info!("[Image-Storage] Cannot find user_status by uid.");
                (
                    Status::BadRequest,
                    Err(Json(ErrorResponse::build(ErrorCode::UserNotExist, ""))),
                )
            }
        },
        Err(e) => {
            error!("[Image-Storage] Database Error: {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}

/// Download image
///
/// ## Parameters
///
/// - `Auth`: Authenticated User
/// - `ReferrerCheck`: Check request header
/// - `Connection<PgDb>`: Postgres connection
/// - `Connection<MinioImageStorage>`: Image storage connection
/// - `&str`: Filename
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `Vec<u8>`: Image as bytes
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///     - `ErrorCode::FileNotExist`
///
#[get("/images/<filename>")]
async fn download_image(
    auth: Auth,
    _ref: ReferrerCheck,
    db: Connection<PgDb>,
    bucket: Connection<MinioImageStorage>,
    filename: &str,
) -> (Status, (ContentType, Result<Vec<u8>, Json<ErrorResponse>>)) {
    info!("[IMAGE] User {} id downloading image.", auth.id);
    // get a file
    let (data, code) = bucket.get_object(filename).await.unwrap();
    match code {
        200 => {
            // update last download time
            let pg_con = db.into_inner();
            let record = image::ActiveModel {
                filename: Set(filename.to_owned()),
                last_download_time: Set(Utc::now().with_timezone(&FixedOffset::east(8 * 3600))),
                ..Default::default()
            };
            let _ = record.update(&pg_con).await;
            match filename.split('.').last().unwrap() {
                "png" => (Status::Ok, (ContentType::PNG, Ok(data))),
                "gif" => (Status::Ok, (ContentType::GIF, Ok(data))),
                _ => (Status::Ok, (ContentType::JPEG, Ok(data))),
            }
        }
        _ => (
            Status::NotFound,
            (
                ContentType::JSON,
                Err(Json(ErrorResponse::build(ErrorCode::FileNotExist, ""))),
            ),
        ),
    }
}

/// Show all the images stored
///
/// ## Parameters
///
/// - `Auth`: Authenticated User
/// - `Connection<MinioImageStorage>`: Image storage connection
///
/// ## Returns
///
/// - `Status`: HTTP status
/// - `Json<Vec<Vec<(String, u64)>>>`: List of storaged image
///
/// ## Errors
///
/// - `ErrorResponse`: Error message
///     - `ErrorCode::DatabaseErr`
///
#[get("/images")]
async fn get_images(
    auth: Auth,
    bucket: Connection<MinioImageStorage>,
) -> (
    Status,
    Result<Json<Vec<Vec<(String, u64)>>>, Json<ErrorResponse>>,
) {
    // list files
    info!("[IMAGE] User {} id fetching image list.", auth.id);
    let bucket_list = bucket.list("/".to_owned(), Some("/".to_owned())).await;
    match bucket_list {
        Ok(list) => {
            let results: Vec<Vec<(String, u64)>> = list
                .iter()
                .map(|item| {
                    let r: Vec<(String, u64)> = item
                        .contents
                        .iter()
                        .map(|c| (c.key.to_owned(), c.size))
                        .collect();
                    r
                })
                .collect();
            (Status::Ok, Ok(Json(results)))
        }
        Err(e) => {
            log::error!("[Image-Storage] Database Error {:?}", e);
            (
                Status::InternalServerError,
                Err(Json(ErrorResponse::default())),
            )
        }
    }
}
