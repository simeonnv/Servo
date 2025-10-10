use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use pingora::ErrorType::HTTPStatus;
use pingora::{Error, Result};
use pingora::{
    prelude::HttpPeer,
    proxy::{ProxyHttp, Session},
};
use servo_toml::tomls::ConfigToml;

use crate::server_map::{DownStreamHost, Server, ServerMap};

pub struct ProxyState {
    pub server_map: ServerMap,
}

pub struct ProxyCTX {
    pub server: Option<Arc<Server>>,
    pub host_header: String,
    pub endpoint: String,
    pub proxy_pass: String,
}

impl Default for ProxyCTX {
    fn default() -> Self {
        Self {
            server: None,
            host_header: "".into(),
            endpoint: "".into(),
            proxy_pass: "".into(),
        }
    }
}

#[async_trait]
impl ProxyHttp for ProxyState {
    type CTX = ProxyCTX;

    fn new_ctx(&self) -> Self::CTX {
        Self::CTX::default()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // ctx.beta_user = check_beta_user(session.req_header());
        let req_header = session.req_header();
        let host_header = req_header
            .headers
            .get("host")
            .ok_or(Error::explain(HTTPStatus(400), "Missing host header"))?
            .to_str()
            .map_err(|err| Error::explain(HTTPStatus(400), format!("Invalid host header: {err}")))?
            .to_owned();

        let endpoint = req_header.uri.path();
        let server = self
            .server_map
            .routes
            .get(&DownStreamHost(host_header.clone()))
            .ok_or(Error::new(HTTPStatus(502)))?
            .clone();

        ctx.host_header = host_header;
        ctx.endpoint = endpoint.to_owned();
        ctx.server = Some(server);

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // most of the logic will prolly be here
        let mut peer = HttpPeer::new(("10.0.0.1", 80), false, "".into());
        peer.options.connection_timeout = Some(Duration::from_millis(100));
        Ok(Box::new(peer))
    }
}
