use std::sync::Arc;

use servo_auth::jwt::{Jwt, algoritms::Rsa};

use crate::server_map::{DownStreamHost, Server, Upstream};

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
    pub host_header: DownStreamHost,
    pub upstream: Arc<Upstream>,
    pub jwt: Option<Jwt<Rsa>>,
}
