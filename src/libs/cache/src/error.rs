use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to init cache connection => {0}")]
    FailedToInitCacheConn(String),

    #[error("failed to build cache connection pool => {0}")]
    FailedToBuildCacheConnectionPool(String),

    #[error("the cache connection is dead => {0}")]
    DisconnectedFromCache(String),
}
