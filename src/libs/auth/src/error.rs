use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database Error: {0}")]
    DBError(#[from] sqlx::Error),

    #[error("Unable to serialize/unserialize jwt : {0}")]
    JWTSerializationError(String),

    #[error("The jwt is invalid: {0}")]
    InvalidJWT(String),

    #[error("Invalid Refresh token: {0}")]
    InvalidRefreshToken(String),

    #[error("Crypto Error: {0}")]
    CryptoError(crypto::Error),

    #[error("Hash Error: {0}")]
    HashError(String), // Adjust type based on crypto::Error::HashError's structure
}

impl From<crypto::Error> for Error {
    fn from(err: crypto::Error) -> Self {
        match err {
            crypto::Error::HashError(err) => Error::HashError(err),
            _ => Error::CryptoError(err),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JWTSerializationError(err.to_string()) // _ => Error::CryptoError(err),
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::InvalidJWT(err.to_string()) // _ => Error::CryptoError(err),
    }
}
