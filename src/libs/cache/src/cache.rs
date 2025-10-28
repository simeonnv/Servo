use r2d2::Pool;
use redis::{Client, Commands};

use crate::Error;

pub struct Cache {
    pub connection_addr: String,
    cache_pool: Pool<Client>,
}

pub enum CacheStatus {
    Hit(Box<[u8]>),
    Miss(String),
}

impl Cache {
    pub fn new(addr: &str) -> Result<Self, Error> {
        let client =
            redis::Client::open(addr).map_err(|e| Error::FailedToInitCacheConn(e.to_string()))?;
        let pool = r2d2::Pool::builder()
            .build(client)
            .map_err(|e| Error::FailedToBuildCacheConnectionPool(e.to_string()))?;

        Ok(Self {
            connection_addr: addr.to_owned(),
            cache_pool: pool,
        })
    }

    pub fn lookup(&self, key: &str) -> Result<CacheStatus, Error> {
        let mut conn = self
            .cache_pool
            .get()
            .map_err(|e| Error::DisconnectedFromCache(e.to_string()))?;

        let output = match conn.get(key) {
            Ok(e) => CacheStatus::Hit(e),
            Err(err) => return Ok(CacheStatus::Miss(err.to_string())),
        };

        Ok(output)
    }
}
