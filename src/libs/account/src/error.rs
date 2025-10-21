use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database Error: {0}")]
    DBError(#[from] sqlx::Error),

    #[error("Cryptography Error: {0}")]
    CryptoError(#[from] servo_crypto::Error),

    #[error("Invalid username or password! Error: {0}")]
    InvalidCredentials(String),

    #[error("Invalid Account! Error: {0}")]
    InvalidAccount(String),

    #[error("Friend request doesnt exist!")]
    FriendRequestDoesntExist(),
}
