use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use crypto::digest::Digest;
use crypto::md5::Md5;
use sea_orm::{entity::*, ActiveModelTrait};

use crate::pgdb::image;
use crate::pool::{MinioImageStorage, PgDb};
use crate::req::storage::{ReferrerCheck, SaveImage};

use chrono::{FixedOffset, Utc};

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
) -> (Status, String) {
    info!("[IMAGE] User {} id uploading image.", auth.id);
    // put a file
    // check content type
    match image.content_type.as_str() {
        "jpg" | "jpeg" | "png" | "gif" => {}
        _ => {
            return (Status::UnsupportedMediaType, "".to_string());
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
            (Status::Ok, filename)
        }
        Ok((_, code)) => (Status::new(code), "".to_string()),
        Err(e) => (Status::InternalServerError, format!("{}", e)),
    }
}

#[get("/image/<filename>")]
async fn download_image(
    auth: Auth,
    _ref: ReferrerCheck,
    db: Connection<PgDb>,
    bucket: Connection<MinioImageStorage>,
    filename: &str,
) -> Result<Vec<u8>, status::NotFound<String>> {
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
            Ok(data)
        }
        _ => Err(status::NotFound(format!("Error code: {}", code))),
    }
}

#[get("/images")]
async fn get_images(auth: Auth, bucket: Connection<MinioImageStorage>) -> Json<Vec<(String, u64)>> {
    // list files
    info!("[IMAGE] User {} id fetching image list.", auth.id);
    let bucket_list = bucket
        .list("/".to_owned(), Some("/".to_owned()))
        .await
        .expect("Can not list");
    let mut results: Vec<(String, u64)> = Vec::new();
    for result in bucket_list {
        for item in result.contents {
            results.push((item.key, item.size));
        }
    }
    Json(results)
}
