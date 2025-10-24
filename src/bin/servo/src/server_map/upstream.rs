use crate::server_map::{ProxyPass, UpstreamAuth};

#[derive(Debug)]
pub struct Upstream {
    pub url_concat_suffix: String,
    pub proxy_pass: ProxyPass,
    pub auth: Option<UpstreamAuth>,
}
