use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use log::{debug, info};
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

#[derive(Debug)]
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
        let endpoint = req_header.uri.path();

        let host_header = match req_header.headers.get("host") {
            Some(e) => e,
            None => {
                info!("Request filtered: no host header");
                return Ok(true);
            }
        };

        let host_header = match host_header.to_str() {
            Ok(e) => e,
            Err(err) => {
                info!("invalid charecters in header: {err}");
                return Ok(true);
            }
        };

        let server = match self.server_map.routes.get(host_header) {
            Some(e) => e,
            None => {
                info!("unable to map the host header to a actual server!");
                return Ok(true);
            }
        };

        let proxy_pass = match server.routes.at(endpoint) {
            Ok(e) => e,
            Err(err) => {
                info!("endpoint / path doesnt map to a upstream / proxy pass: {err}");
                return Ok(true);
            }
        };

        ctx.server = Some(server.clone());
        ctx.host_header = host_header.to_owned();
        ctx.proxy_pass = proxy_pass.value.0.clone();
        ctx.endpoint = endpoint.to_owned();

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let mut peer = HttpPeer::new(&ctx.proxy_pass, false, "".into());
        peer.options.connection_timeout = Some(Duration::from_millis(100));
        Ok(Box::new(peer))
    }
}
