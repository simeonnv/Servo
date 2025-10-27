use std::{collections::HashSet, sync::Arc};

use crate::server_map::{ProxyPass, RateLimiter, UpstreamAuth};

#[derive(Debug)]
pub struct Upstream {
    pub url_concat_suffix: String,
    pub proxy_pass: ProxyPass,
    pub auth: Option<UpstreamAuth>,
    pub blacklisted_endpoints: HashSet<String>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
}
