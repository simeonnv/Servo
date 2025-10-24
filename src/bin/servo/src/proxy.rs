use std::time::Duration;

use async_trait::async_trait;
use http::Uri;
use log::{debug, error, info, warn};
use pingora::ErrorType::HTTPStatus;
use pingora::http::RequestHeader;
use pingora::{Error, Result};
use pingora::{
    prelude::HttpPeer,
    proxy::{ProxyHttp, Session},
};

use crate::proxy_ctx::{AfterFilterCTX, ProxyCTX};
use crate::server_map::{DownStreamHost, ServerMap};

pub struct Proxy {
    pub server_map: ServerMap,
}

#[async_trait]
impl ProxyHttp for Proxy {
    type CTX = ProxyCTX;

    fn new_ctx(&self) -> Self::CTX {
        Self::CTX::default()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // ctx.beta_user = check_beta_user(session.req_header());
        let req_header = session.req_header();
        let endpoint = req_header.uri.path();

        let host_header = match DownStreamHost::try_from(req_header) {
            Ok(e) => e,
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

        let route_match = match server.routes.at(endpoint) {
            Ok(e) => e,
            Err(err) => {
                info!("endpoint / path doesnt map to a upstream / proxy pass: {err}");
                return Ok(true);
            }
        };
        let upstream = route_match.value.clone();

        let after_filter_ctx = AfterFilterCTX {
            server: server.clone(),
            host_header: host_header,
            upstream,
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
        // after_filter_ctx.

        let proxy_pass = after_filter_ctx
            .upstream
            .proxy_pass
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

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        let path = request.uri.path();
        let ctx_after_filter = ctx.after_filter.as_ref().unwrap();
        let url_concat_suffix = &ctx_after_filter.upstream.url_concat_suffix;

        let concat_path = concat_path(path, url_concat_suffix);
        let uri = Uri::builder().path_and_query(concat_path).build().unwrap();

        debug!("routed from path: {path}, to {uri}");

        request.set_uri(uri);

        let upstream_auth = match &ctx_after_filter.upstream.auth {
            Some(e) => e,
            None => return Ok(()),
        };

        if !upstream_auth.jwt_required {
            return Ok(());
        }

        let auth_header = request
            .headers
            .get("Authorization")
            .ok_or_else(|| Error::explain(HTTPStatus(401), "Unauthorized"))?
            .to_str()
            .map_err(|_| Error::explain(HTTPStatus(401), "Unauthorized"))?;

        let jwt = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| Error::explain(HTTPStatus(401), "Unauthorized"))?
            .trim();

        Ok(())
    }

    async fn logging(
        &self,
        session: &mut Session,
        err: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let response_code = session
            .response_written()
            .map_or(0, |resp| resp.status.as_u16());

        let addr = session
            .client_addr()
            .map(|e| e.to_string())
            .unwrap_or("unknown".to_string());

        if let Some(err) = err {
            warn!("{err}");
        }

        info!(
            "{} response code: {response_code}, addr: {}",
            self.request_summary(session, ctx),
            addr
        );
    }
}

fn concat_path(path: &str, suffix: &str) -> String {
    if path == suffix {
        return "/".to_string();
    }
    if path.starts_with(suffix) {
        let rest = path.strip_prefix(suffix).unwrap_or("");
        if rest.starts_with("/") {
            format!("{}", rest)
        } else {
            format!("/{}", rest)
        }
    } else {
        path.to_string()
    }
}
