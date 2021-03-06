//! Models of storage

use rocket::data::{self, Data, FromData, ToByteUnit};
use rocket::http::{ContentType, Status};
use rocket::outcome::Outcome::*;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use regex::Regex;

/// Request guard for save image
///
/// ## Fields
///
/// - `content_type`: String, the converted content type of the image
/// - `content`: Vec<u8>, the content of the image
#[derive(Deserialize)]
pub struct SaveImage {
    pub content_type: ImageContentType,
    pub content: Vec<u8>,
}

/// Allowed types of image
#[derive(Serialize, Deserialize)]
pub enum ImageContentType {
    JPEG,
    PNG,
    GIF,
    JPG,
}

impl std::fmt::Display for ImageContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageContentType::JPEG => write!(f, "jpeg"),
            ImageContentType::PNG => write!(f, "png"),
            ImageContentType::GIF => write!(f, "gif"),
            ImageContentType::JPG => write!(f, "jpg"),
        }
    }
}

/// Error type of saving image
///
/// ## Fields
///
/// - `ImageError::TooLarge`: The image is too large, more than the limit
/// - `ImageError::InvalidType`: The content type is invalid
/// - `ImageError::Io`: Io error
#[derive(Debug)]
pub enum ImageError {
    TooLarge,
    InvalidType,
    Io(std::io::Error),
}

/// Error type for referer check
///
/// ## Fields
///
/// - `RefererError::Empty`: The referer is empty
/// - `RefererError::Invalid`: The referer is invalid
#[derive(Debug)]
pub enum ReferrerError {
    Empty,
    Invalid,
}

/// Request guard for referer check
pub struct ReferrerCheck {}

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
                    ImageContentType::JPEG
                } else if req_ct == &jpg_ct {
                    ImageContentType::JPG
                } else if req_ct == &png_ct {
                    ImageContentType::PNG
                } else if req_ct == &gif_ct {
                    ImageContentType::GIF
                } else {
                    return Failure((Status::UnsupportedMediaType, ImageError::InvalidType));
                }
            }
            _ => return Failure((Status::UnsupportedMediaType, ImageError::InvalidType)),
        };

        // Use a configured limit with name 'person' or fallback to default.
        let limit = req
            .limits()
            .get("minio-image")
            .unwrap_or_else(|| 1.mebibytes());

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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ReferrerCheck {
    type Error = ReferrerError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        fn valid_url(url: &str) -> bool {
            lazy_static! {
                static ref REFERRER: Regex =
                    Regex::new(r"^https://.*(\.)?thuburrow\.com/").unwrap();
            }
            REFERRER.is_match(url)
        }
        // Check referrer in header
        let header_map = req.headers();
        if header_map.contains("Referer") {
            match header_map.get_one("Referer") {
                Some(url) if valid_url(url) => request::Outcome::Success(ReferrerCheck {}),
                _ => Failure((Status::Forbidden, ReferrerError::Invalid)),
            }
        } else {
            Failure((Status::Forbidden, ReferrerError::Empty))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type() {
        assert_eq!(ImageContentType::JPEG.to_string(), "jpeg");
        assert_eq!(ImageContentType::PNG.to_string(), "png");
        assert_eq!(ImageContentType::GIF.to_string(), "gif");
        assert_eq!(ImageContentType::JPG.to_string(), "jpg");
    }
}
