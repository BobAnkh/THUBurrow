use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorCode {
    /// 500 InternalServerError
    DatabaseErr,
    /// 401 Unauthorized
    AuthTokenMissing,
    /// 401 Unauthorized
    AuthTokenInvalid,
    /// 404 NotFound
    BurrowNotExist,
    /// 403 Forbidden
    BurrowNumLimit,
    /// 403 Forbidden
    BurrowInvalid,
    /// 404 NotFound
    PostNotExist,
    /// 404 NotFound
    ReplyNotExist,
    /// 400 BadRequest
    EmailInvalid,
    /// 400 BadRequest
    EmailDuplicate,
    /// 400 BadRequest
    UsernameDuplicate,
    /// 429 TooManyRequests
    RateLimit,
    /// 403 Forbidden
    UserForbidden,
    ///400 BadRequest
    UserNotExist,
    /// 400 BadRequest
    EmptyField,
    /// 400 BadRequest
    SectionInvalid,
    /// 400 BadRequest
    CredentialInvalid,
    /// 415 UnsupportedMediaType
    UnsupportedMediaType,
    /// 404 NotFound
    FileNotExist,
    /// 500 InternalServerError
    Unknown,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub error: ErrorMessage,
}

impl ErrorResponse {
    pub fn new() -> Self {
        ErrorResponse {
            error: ErrorMessage {
                code: ErrorCode::DatabaseErr,
                message: String::from(""),
            },
        }
    }

    pub fn build<T: Into<String>>(code: ErrorCode, message: T) -> Self {
        ErrorResponse {
            error: ErrorMessage {
                code,
                message: message.into(),
            },
        }
    }
}

impl Default for ErrorResponse {
    fn default() -> Self {
        Self::new()
    }
}
