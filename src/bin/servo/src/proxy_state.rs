use std::time::Duration;

use async_trait::async_trait;
use pingora::Result;
use pingora::{
    prelude::HttpPeer,
    proxy::{ProxyHttp, Session},
};
use servo_toml::tomls::ConfigToml;

pub struct ProxyState {
    pub config: ConfigToml,
}

#[async_trait]
impl ProxyHttp for ProxyState {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {
        Self::CTX::default()
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // most of the logic will prolly be here
        let mut peer = HttpPeer::new(("10.0.0.1", 80), false, "".into());
        peer.options.connection_timeout = Some(Duration::from_millis(100));
        Ok(Box::new(peer))
    }
}
