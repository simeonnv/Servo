use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to generate key pair: {0}")]
    KeyGenerateError(String),

    #[error("invalid public/private key: {0}")]
    InvalidKeyError(String),

    #[error("invalid algorithm for given key: {0}")]
    AlgorithmError(String),

    #[error("invalid signiture for given key: {0}")]
    InvalidSignitureError(String),

    #[error("Error while encrypting data: {0}")]
    EncryptionError(String),

    #[error("Error while decrypting data: {0}")]
    DecryptionError(String),

    #[error("The cipthertext is invalid!: {0}")]
    InvalidCipthertextError(String),

    #[error("Error while Hashing!: {0}")]
    HashError(String),

    #[error("Unknown: {0}")]
    Unknownr(String),
}

impl From<argon2::Error> for Error {
    fn from(err: argon2::Error) -> Self {
        Error::HashError(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(err: argon2::password_hash::Error) -> Self {
        Error::HashError(err.to_string())
    }
}

// impl From<aes_gcm::Error> for Error {
//     fn from(err: aes_gcm::Error) -> Self {
//         let nz = aes_gcm::Error;
//         match err {
//             aes_gcm::Error -> Error::EncryptionError(err),
//         }
//     }
// }
