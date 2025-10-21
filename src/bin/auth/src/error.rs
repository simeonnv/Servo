use actix_web::{HttpResponse, ResponseError};
use log::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Internal server error!: {0}")]
    Internal(String),

    #[error("Bad request!: {0}")]
    BadRequest(String),

    #[error("Unauthorized!: {0}")]
    Unauthorized(String),

    #[error("Too many requests!: {0}")]
    Conflict(String),

    #[error("Too many requests!: {0}")]
    TooManyRequests(String),

    #[error("Not found")]
    NotFound(),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::Conflict(msg) => HttpResponse::Conflict().body(msg.to_string()),
            Error::Unauthorized(msg) => HttpResponse::Unauthorized().body(msg.to_string()),
            Error::BadRequest(msg) => HttpResponse::BadRequest().body(msg.to_string()),
            Error::Internal(msg) => HttpResponse::InternalServerError().body(msg.to_string()),
            Error::TooManyRequests(msg) => HttpResponse::TooManyRequests().body(msg.to_string()),
            Error::NotFound() => HttpResponse::NotFound().finish(),
        }
    }
}

impl From<servo_crypto::Error> for Error {
    fn from(err: servo_crypto::Error) -> Self {
        match err {
            servo_crypto::Error::InvalidCipthertextError(e) => Error::Unauthorized(e),
            servo_crypto::Error::InvalidSignitureError(e) => Error::Unauthorized(e),
            servo_crypto::Error::DecryptionError(e) => Error::Unauthorized(e),
            _ => Error::Internal(err.to_string()),
        }
    }
}

impl From<servo_auth::Error> for Error {
    fn from(err: servo_auth::Error) -> Self {
        match err {
            servo_auth::Error::InvalidJWT(e) => Error::Unauthorized(e),
            servo_auth::Error::InvalidRefreshToken(e) => Error::Unauthorized(e),
            // auth::Error::InvalidCredentials(e) => Error::BadRequest(e),
            // auth::Error::InvalidAccount(e) => Error::BadRequest(e),
            _ => Error::Internal(err.to_string()),
        }
    }
}

impl From<servo_account::Error> for Error {
    fn from(err: servo_account::Error) -> Self {
        match err {
            servo_account::Error::InvalidCredentials(e) => Error::BadRequest(e),
            servo_account::Error::InvalidAccount(e) => Error::BadRequest(e),
            servo_account::Error::FriendRequestDoesntExist() => {
                Error::BadRequest("Friend request does not exist!".into())
            }
            _ => Error::Internal(err.to_string()),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        error!("db error: {err}");
        Error::BadRequest("Internal server error!".into())
    }
}
