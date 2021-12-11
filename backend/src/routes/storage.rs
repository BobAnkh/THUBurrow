use chrono::{FixedOffset, Utc};
use crypto::digest::Digest;
use crypto::md5::Md5;
use rocket::http::{Status, ContentType};
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;
use sea_orm::{entity::*, ActiveModelTrait};

use crate::models::error::*;
use crate::models::storage::{ReferrerCheck, SaveImage};
use crate::pgdb::image;
use crate::pool::{MinioImageStorage, PgDb};
use crate::utils::auth::Auth;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/storage",
        routes![upload_image, download_image, get_images],
    )
}

#[post("/images", data = "<image>")]
async fn upload_image(
    auth: Auth,
    db: Connection<PgDb>,
    bucket: Connection<MinioImageStorage>,
    image: SaveImage,
) -> (Status, Result<String, Json<ErrorResponse>>) {
    info!("[IMAGE] User {} id uploading image.", auth.id);
    // put a file
    // check content type
    match image.content_type.as_str() {
        "jpg" | "jpeg" | "png" | "gif" => {}
        _ => {
            return (
                Status::UnsupportedMediaType,
                Err(Json(ErrorResponse::build(
                    ErrorCode::UnsupportedMediaType,
                    "",
                ))),
            );
        }
    }
    let mut hash_md5 = Md5::new();
    hash_md5.input(image.content.as_slice());
    let filename = hash_md5.result_str() + "." + &image.content_type;
    let image_size = image.content.len() as i32;
    match bucket
        .put_object(filename.as_str(), image.content.as_slice())
        .await
    {
        Ok((_, 200)) => {
            let now = Utc::now().with_timezone(&FixedOffset::east(8 * 3600));
            let record = image::ActiveModel {
                filename: Set(filename.to_owned()),
                user_id: Set(auth.id),
                size: Set(image_size),
                create_time: Set(now.to_owned()),
                last_download_time: Set(now),
            };
            match record.insert(&db.into_inner()).await {
                Ok(_) => {
                    log::info!("[Image-Storage] Add image");
                }
                Err(e) => {
                    log::warn!("[Image-Storage] Same image: {}", e);
                }
            }
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

#[get("/image/<filename>")]
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
            (Status::Ok, (ContentType::JPEG, Ok(data)))
        }
        _ => (
            Status::NotFound,
            (ContentType::JSON, Err(Json(ErrorResponse::build(ErrorCode::FileNotExist, "")))),
        ),
    }
}

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
