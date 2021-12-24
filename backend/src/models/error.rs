//! Error Response of all the public api interfaces

use serde::{Deserialize, Serialize};

/// ErrorCode for all the public interfaces
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// Body of ErrorResponse
///
/// ## Fields
///
/// - `code`: ErrorCode, the summary of the error
/// - `message`: String, the detail of the error
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ErrorMessage {
    pub code: ErrorCode,
    pub message: String,
}

/// Error Response for all the public interfaces
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ErrorResponse {
    pub error: ErrorMessage,
}

impl ErrorResponse {
    /// Default ErrorResponse
    ///
    /// Returns the DatabaseErr, which is the mostly used case.
    pub fn new() -> Self {
        ErrorResponse {
            error: ErrorMessage {
                code: ErrorCode::DatabaseErr,
                message: String::from(""),
            },
        }
    }

    /// ErrorResponse with ErrorCode and Message
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_error() {
        let error = ErrorResponse::new();
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::DatabaseErr,
                    message: String::from(""),
                },
            }
        );
        let error = ErrorResponse::default();
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::DatabaseErr,
                    message: String::from(""),
                },
            }
        );
    }

    #[test]
    fn test_all_kind_error() {
        let error = ErrorResponse::build(ErrorCode::AuthTokenMissing, "AuthTokenMissing");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::AuthTokenMissing,
                    message: String::from("AuthTokenMissing"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::AuthTokenInvalid, "AuthTokenInvalid");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::AuthTokenInvalid,
                    message: String::from("AuthTokenInvalid"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::BurrowNotExist, "BurrowNotExist");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::BurrowNotExist,
                    message: String::from("BurrowNotExist"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::BurrowNumLimit, "BurrowNumLimit");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::BurrowNumLimit,
                    message: String::from("BurrowNumLimit"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::BurrowInvalid, "BurrowInvalid");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::BurrowInvalid,
                    message: String::from("BurrowInvalid"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::PostNotExist, "PostNotExist");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::PostNotExist,
                    message: String::from("PostNotExist"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::ReplyNotExist, "ReplyNotExist");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::ReplyNotExist,
                    message: String::from("ReplyNotExist"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::EmailInvalid, "EmailInvalid");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::EmailInvalid,
                    message: String::from("EmailInvalid"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::EmailDuplicate, "EmailDuplicate");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::EmailDuplicate,
                    message: String::from("EmailDuplicate"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::UsernameDuplicate, "UsernameDuplicate");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::UsernameDuplicate,
                    message: String::from("UsernameDuplicate"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::RateLimit, "RateLimit");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::RateLimit,
                    message: String::from("RateLimit"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::UserForbidden, "UserForbidden");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::UserForbidden,
                    message: String::from("UserForbidden"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::UserNotExist, "UserNotExist");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::UserNotExist,
                    message: String::from("UserNotExist"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::EmptyField, "EmptyField");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::EmptyField,
                    message: String::from("EmptyField"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::SectionInvalid, "SectionInvalid");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::SectionInvalid,
                    message: String::from("SectionInvalid"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::CredentialInvalid, "CredentialInvalid");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::CredentialInvalid,
                    message: String::from("CredentialInvalid"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::UnsupportedMediaType, "UnsupportedMediaType");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::UnsupportedMediaType,
                    message: String::from("UnsupportedMediaType"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::FileNotExist, "FileNotExist");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::FileNotExist,
                    message: String::from("FileNotExist"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::Unknown, "Unknown");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::Unknown,
                    message: String::from("Unknown"),
                },
            }
        );
        let error = ErrorResponse::build(ErrorCode::None, "");
        assert_eq!(
            error,
            ErrorResponse {
                error: ErrorMessage {
                    code: ErrorCode::None,
                    message: String::from(""),
                },
            }
        );
    }
}
