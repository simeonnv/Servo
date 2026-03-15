use std::any::Any;
use std::time::Duration;

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use fred::prelude::{Client, KeysInterface};
use fred::types::Expiration;
use pingora::ErrorType::Custom;
use pingora::cache::key::{CacheHashKey, CompactCacheKey, HashBinary};
use pingora::cache::storage::{HandleHit, HandleMiss, MissFinishType};
use pingora::cache::{CacheKey, Storage, trace::SpanHandle};
use pingora::cache::{CacheMeta, HitHandler, MissHandler, PurgeType};
use pingora::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RedisCache {
    redis_pool: Client,
}

impl RedisCache {
    pub fn new(redis_pool: Client) -> Self {
        Self { redis_pool }
    }
}

#[derive(Debug)]
pub struct RedisMissHandler {
    client: Client,
    meta: (Bytes, Bytes),
    key: HashBinary,
    body_buf: BytesMut,
}

#[derive(Debug)]
pub struct RedisHitHandler {
    pub done: bool,
    pub obj: RedisCacheObject,
}
impl RedisHitHandler {
    pub fn new(obj: RedisCacheObject) -> Self {
        Self { done: false, obj }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisCacheObject {
    body: Bytes,
    meta: (Bytes, Bytes),
    len: usize,
}

#[async_trait]
impl HandleMiss for RedisMissHandler {
    async fn write_body(&mut self, data: bytes::Bytes, _eof: bool) -> Result<()> {
        log::info!("redis miss write body");
        self.body_buf.extend_from_slice(&data);
        Ok(())
    }

    async fn finish(self: Box<Self>) -> Result<MissFinishType> {
        log::info!("redis miss finish");
        let body = self.body_buf.to_owned().freeze();
        let size = body.len() + self.meta.0.len() + self.meta.1.len();
        let obj = RedisCacheObject {
            body,
            meta: self.meta.to_owned(),
            len: size,
        };
        let obj: Vec<u8> = postcard::to_allocvec(&obj)
            .map_err(|_| Error::new(Custom("EncodeRedisPostcardObj")))?;

        let _: Option<()> = self
            .client
            .set(
                &self.key[..],
                obj,
                Some(Expiration::EX(60 * 60 * 16)),
                None,
                false,
            )
            .await
            .ok();

        log::info!("redis miss created");
        Ok(MissFinishType::Created(size))
    }
}

#[async_trait]
impl HandleHit for RedisHitHandler {
    async fn read_body(&mut self) -> Result<Option<Bytes>> {
        log::info!("redis hit read body");
        if self.done {
            return Ok(None);
        }
        self.done = true;
        Ok(Some(self.obj.body.to_owned()))
    }

    async fn finish(
        mut self: Box<Self>,
        _storage: &'static (dyn Storage + Sync),
        _key: &CacheKey,
        _trace: &SpanHandle,
    ) -> Result<()> {
        log::info!("redis hit finish");
        Ok(())
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }
}

#[async_trait]
impl Storage for RedisCache {
    async fn lookup(
        &'static self,
        key: &CacheKey,
        _trace: &SpanHandle,
    ) -> Result<Option<(CacheMeta, HitHandler)>> {
        log::info!("redis lookup");
        let key = key.combined_bin();
        let res = match self.redis_pool.get::<Option<Vec<u8>>, _>(&key[..]).await {
            Ok(Some(e)) => {
                log::info!("redis found");
                e
            }
            _ => {
                log::info!("redis not found");
                return Ok(None);
            }
        };
        let obj: RedisCacheObject =
            postcard::from_bytes(&res).map_err(|_| Error::new(Custom("DecodeRedisPostcardObj")))?;

        let meta = CacheMeta::deserialize(&obj.meta.0, &obj.meta.1)?;

        log::info!("redis calling hit handler");
        Ok(Some((meta, Box::new(RedisHitHandler::new(obj)))))
    }

    async fn get_miss_handler(
        &'static self,
        key: &CacheKey,
        meta: &CacheMeta,
        _trace: &SpanHandle,
    ) -> Result<MissHandler> {
        log::info!("redis calling miss handler");
        let key = key.combined_bin();

        let raw_meta = meta.serialize()?;
        let meta = (Bytes::from(raw_meta.0), Bytes::from(raw_meta.1));

        let miss_handler = RedisMissHandler {
            client: self.redis_pool.to_owned(),
            meta,
            key,
            body_buf: BytesMut::new(),
        };
        Ok(Box::new(miss_handler))
    }

    async fn purge(
        &'static self,
        key: &CompactCacheKey,
        purge_type: PurgeType,
        trace: &SpanHandle,
    ) -> Result<bool> {
        log::info!("redis purge");
        let key = key.combined_bin();
        Ok(self.redis_pool.del::<(), _>(&key[..]).await.is_ok())
    }

    async fn update_meta(
        &'static self,
        key: &CacheKey,
        meta: &CacheMeta,
        trace: &SpanHandle,
    ) -> Result<bool> {
        log::info!("redis update meta");
        let key = key.combined_bin();

        let new_meta = meta.serialize()?;
        let new_meta = (Bytes::from(new_meta.0), Bytes::from(new_meta.1));

        let old_res = match self.redis_pool.get::<Option<Vec<u8>>, _>(&key[..]).await {
            Ok(Some(e)) => e,
            _ => return Ok(false),
        };
        log::info!("redis update meta fetched old");

        let new_obj: RedisCacheObject = {
            let mut old_obj: RedisCacheObject = postcard::from_bytes(&old_res)
                .map_err(|_| Error::new(Custom("DecodeRedisPostcardObj")))?;
            old_obj.meta = new_meta;
            old_obj
        };

        let new_obj: Vec<u8> = postcard::to_allocvec(&new_obj)
            .map_err(|e| Error::new(Custom("EncodeRedisPostcardObj")))?;

        let _: Option<()> = self
            .redis_pool
            .set(&key[..], new_obj, None, None, false)
            .await
            .ok();
        log::info!("redis update meta inserted new");

        Ok(true)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
        self
    }
}
