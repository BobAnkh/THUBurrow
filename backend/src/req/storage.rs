use rocket::data::{self, Data, FromData, ToByteUnit};
use rocket::http::{ContentType, Status};
use rocket::outcome::Outcome::*;
use rocket::request::Request;
use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SaveImage {
    pub content_type: String,
    pub content: Vec<u8>,
}

#[derive(Serialize)]
pub struct SaveImageResponse {
    pub name: String,
    pub success: bool,
    pub error: String,
}

#[derive(Debug)]
pub enum ImageError {
    TooLarge,
    InvalidType,
    Io(std::io::Error),
}

#[rocket::async_trait]
impl<'r> FromData<'r> for SaveImage {
    type Error = ImageError;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        // Ensure the content type is correct before opening the data.
        let jpeg_ct = ContentType::new("image", "jpeg");
        let jpg_ct = ContentType::new("image", "jpg");
        let png_ct = ContentType::new("image", "png");
        let gif_ct = ContentType::new("image", "gif");
        let content_type = match req.content_type() {
            Some(req_ct) => {
                if req_ct == &jpeg_ct {
                    "jpeg".to_string()
                } else if req_ct == &jpg_ct {
                    "jpg".to_string()
                } else if req_ct == &png_ct {
                    "png".to_string()
                } else if req_ct == &gif_ct {
                    "gif".to_string()
                } else {
                    return Failure((Status::UnsupportedMediaType, ImageError::InvalidType));
                }
            }
            _ => return Failure((Status::UnsupportedMediaType, ImageError::InvalidType)),
        };

        // Use a configured limit with name 'person' or fallback to default.
        let limit = req.limits().get("minio-image").unwrap_or(1.mebibytes());

        // Read the data into a string.
        let content = match data.open(limit).into_bytes().await {
            Ok(payload) if payload.is_complete() => payload.into_inner(),
            Ok(_) => return Failure((Status::PayloadTooLarge, ImageError::TooLarge)),
            Err(e) => return Failure((Status::InternalServerError, ImageError::Io(e))),
        };

        // Return the data.
        Success(SaveImage {
            content_type,
            content,
        })
    }
}
