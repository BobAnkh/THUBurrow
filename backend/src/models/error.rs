use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorCode {
    DatabaseErr,
    AuthTokenMissing,
    AuthTokenInvalid,
    BurrowNotExist,
    BurrowNumLimit,
    BurrowInvalid,
    PostNotExist,
    ReplyNotExist,
    EmailInvalid,
    EmailDuplicate,
    RateLimit,
    UserForbidden,
    UserNotExist,
    EmptyField,
    WrongField,
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
