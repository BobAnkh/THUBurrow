use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorCode {
    DatabaseErr,
    AuthTokenMissing,
    AuthTokenInvalid,
    BurrowNotExist,
    PostNotExist,
    ReplyNotExist,
    SearchEmptyTag,
    InvalidEmail,
    DuplicateEmail,
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
                code: ErrorCode::None,
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
