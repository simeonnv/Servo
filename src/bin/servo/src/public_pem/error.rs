use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch public pem => {0}")]
    FailedToFetchPublicPem(#[from] reqwest::Error),

    #[error("failed to read public pem from fs => {0}")]
    FailedToReadPublicPemFromFS(String),

    #[error("failed to fetch public pem bc error code is other than 200 => {0}")]
    FailedToFetchPublicPemFromNot200(String),
}
