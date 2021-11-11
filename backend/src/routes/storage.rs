use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_db_pools::Connection;

use crypto::digest::Digest;
use crypto::md5::Md5;

use crate::pool::MinioImageStorage;
use crate::req::storage::{ReferrerCheck, SaveImage};

use crate::utils::sso::SsoAuth;

pub async fn init(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/storage",
        routes![upload_image, download_image, get_images],
    )
}

#[post("/images", data = "<image>")]
async fn upload_image(
    auth: SsoAuth,
    bucket: Connection<MinioImageStorage>,
    image: SaveImage,
) -> (Status, Option<String>) {
    info!("[IMAGE] User {} id uploading image.", auth.id);
    // put a file
    // check content type
    match image.content_type.as_str() {
        "jpg" | "jpeg" | "png" | "gif" => {}
        _ => {
            return (Status::UnsupportedMediaType, None);
        }
    }
    let mut hash_md5 = Md5::new();
    hash_md5.input(image.content.as_slice());
    let filename = hash_md5.result_str() + "." + &image.content_type;
    match bucket
        .put_object(filename.as_str(), image.content.as_slice())
        .await
    {
        Ok((_, 200)) => (Status::Ok, Some(filename)),
        Ok((_, code)) => (Status::new(code), None),
        Err(e) => (Status::InternalServerError, Some(format!("{}", e))),
    }
}

#[get("/image/<filename>")]
async fn download_image(
    auth: SsoAuth,
    _ref: ReferrerCheck,
    bucket: Connection<MinioImageStorage>,
    filename: &str,
) -> Result<Vec<u8>, status::NotFound<String>> {
    info!("[IMAGE] User {} id downloading image.", auth.id);
    // get a file
    let (data, code) = bucket.get_object(filename).await.unwrap();

    match code {
        200 => Ok(data),
        _ => Err(status::NotFound(format!("Error code: {}", code))),
    }
}

#[get("/images")]
async fn get_images(
    auth: SsoAuth,
    bucket: Connection<MinioImageStorage>,
) -> Json<Vec<(String, u64)>> {
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
