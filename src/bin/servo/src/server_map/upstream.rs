use std::{collections::HashSet, sync::Arc};

use fred::prelude::Client;

use crate::{
    redis_cache::RedisCache,
    server_map::{ProxyPass, RateLimiter, UpstreamAuth},
};

#[derive(Debug)]
pub struct Upstream {
    pub url_concat_suffix: String,
    pub proxy_pass: ProxyPass,
    pub auth: Option<UpstreamAuth>,
    pub cache: Option<UpstreamCache>,
    pub blacklisted_endpoints: HashSet<String>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
}

pub struct UpstreamCache {
    pub cache: &'static RedisCache,
    pub cache_time_secs: u64,
}
