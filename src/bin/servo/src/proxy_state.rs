use std::time::Duration;

use async_trait::async_trait;
use log::{error, info};
use pingora::ErrorType::HTTPStatus;
use pingora::{Error, Result};
use pingora::{
    prelude::HttpPeer,
    proxy::{ProxyHttp, Session},
};
use servo_types::DownStreamHost;

use crate::proxy_ctx::{AfterFilterCTX, ProxyCTX};
use crate::server_map::ServerMap;

pub struct ProxyState {
    pub server_map: ServerMap,
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

        let host_header = match DownStreamHost::try_from(req_header) {
            Ok(e) => e.into_owned_host(),
            Err(err) => {
                info!("unable to parse DownStreamHost => {err}");
                return Ok(true);
            }
        };

        let server = match self.server_map.routes.get(&host_header) {
            Some(e) => e,
            None => {
                info!("unable to map the host header to a actual server!");
                return Ok(true);
            }
        };

        let proxy_passes = match server.routes.at(endpoint) {
            Ok(e) => e,
            Err(err) => {
                info!("endpoint / path doesnt map to a upstream / proxy pass: {err}");
                return Ok(true);
            }
        };

        let after_filter_ctx = AfterFilterCTX {
            server: server.clone(),
            endpoint: endpoint.to_owned(),
            host_header: host_header,
            proxy_passes: proxy_passes.value.to_owned(),
        };

        ctx.after_filter = Some(after_filter_ctx);

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let after_filter_ctx = ctx.after_filter.as_ref().unwrap();

        let proxy_pass = after_filter_ctx
            .proxy_passes
            .load_balancer
            .select(b"", 256)
            .ok_or_else(|| {
                error!("falied to select proxypass / backend / upstream");
                Error::explain(HTTPStatus(500), "Server is unavailable")
            })?;

        let mut peer = HttpPeer::new(&proxy_pass, false, "".into());
        peer.options.connection_timeout = Some(Duration::from_millis(100));
        Ok(Box::new(peer))
    }
}
