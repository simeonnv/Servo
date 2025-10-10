use crate::server_map::{ProxyPasses, Server};
use std::sync::Arc;

#[derive(Debug)]
pub struct ProxyCTX {
    pub server: Option<Arc<Server>>,
    pub host_header: String,
    pub endpoint: String,
    pub proxy_passes: Option<ProxyPasses>,
}

impl Default for ProxyCTX {
    fn default() -> Self {
        Self {
            server: None,
            host_header: "".into(),
            endpoint: "".into(),
            proxy_passes: None,
        }
    }
}
