use servo_types::{DownStreamHost, ProxyPass, Server};
use std::sync::Arc;

#[derive(Debug)]
pub struct ProxyCTX {
    pub after_filter: Option<AfterFilterCTX>,
}

impl ProxyCTX {
    pub fn new() -> Self {
        Self { after_filter: None }
    }
}

impl Default for ProxyCTX {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct AfterFilterCTX {
    pub server: Arc<Server>,
    pub host_header: DownStreamHost<'static>,
    pub endpoint: String,
    pub proxy_passes: ProxyPass,
}
