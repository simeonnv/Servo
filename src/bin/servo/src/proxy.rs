use crate::jwt_authorize;
use crate::proxy_ctx::{AfterFilterCTX, ProxyCTX};
use crate::server_map::{DownStreamHost, ServerMap};
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
use std::time::Duration;

pub struct Proxy {
    pub server_map: ServerMap,
}

#[async_trait]
impl ProxyHttp for Proxy {
    type CTX = ProxyCTX;

    fn new_ctx(&self) -> Self::CTX {
        Self::CTX::default()
    }

    // gets the reqheader and chooses a server relating to it
    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // ctx.beta_user = check_beta_user(session.req_header());
        let req_header = session.req_header();
        let endpoint = req_header.uri.path();
        // let nz = session.;s
        // dbg!(nz);

        let downstream_ip = match session.client_addr().map(|e| e.as_inet()) {
            Some(Some(e)) => e.ip(),
            _ => {
                debug!("there is no ip/valid ip in header");
                return Ok(true);
            }
        };

        let host_header = match DownStreamHost::try_from(req_header) {
            Ok(e) => e,
            Err(err) => {
                debug!("unable to parse DownStreamHost => {err}");
                return Ok(true);
            }
        };

        let server = match self.server_map.routes.get(&host_header) {
            Some(e) => e,
            None => {
                debug!("unable to map the host header to a actual server!");
                return Ok(true);
            }
        };

        let route_match = match server.routes.at(endpoint) {
            Ok(e) => e,
            Err(err) => {
                debug!("endpoint / path doesnt map to a upstream / proxy pass: {err}");
                return Ok(true);
            }
        };
        let upstream = route_match.value.clone();

        if let Some(rate_limiter) = &upstream.rate_limiter
            && rate_limiter.rate_limit(&downstream_ip)
        {
            debug!("request blocked bc ip: {downstream_ip} is ratelimited");
            return Ok(true);
        }

        if upstream.blacklisted_endpoints.contains(endpoint) {
            debug!("request blocked bc endpoint is in the blacklist!");
            return Ok(true);
        }

        let after_filter_ctx = AfterFilterCTX {
            server: server.clone(),
            host_header: host_header,
            upstream,
        };

        ctx.after_filter = Some(after_filter_ctx);

        Ok(false)
    }

    // extracts a good proxy pass from the load balancer
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let after_filter_ctx = ctx.after_filter.as_ref().unwrap();

        let proxy_pass = after_filter_ctx
            .upstream
            .proxy_pass
            .load_balancer
            .select(b"", 256)
            .ok_or_else(|| {
                error!("failed to select proxypass / backend / upstream");
                Error::explain(HTTPStatus(500), "Server is unavailable")
            })?;

        let mut peer = HttpPeer::new(&proxy_pass, false, "".into());
        peer.options.connection_timeout = Some(Duration::from_millis(100));

        Ok(Box::new(peer))
    }

    // handles route stripping and token auth
    // checks if the token has exp and roles
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

        if let Some(upstream_auth) = &ctx_after_filter.upstream.auth
            && upstream_auth.jwt_required
        {
            let jwt = jwt_authorize(request, upstream_auth).map_err(|err| {
                info!("jwt error: {err}");
                Error::explain(HTTPStatus(401), "Unauthorized")
            })?;

            use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
            use serde_json::Value;
            use std::str::FromStr;
            fn add_json_to_headers(headers: &mut HeaderMap, body: &Value) {
                if let Some(obj) = body.as_object() {
                    for (key, val) in obj {
                        if let Ok(header_name) = HeaderName::from_str(key) {
                            let val_str = match val {
                                Value::String(s) => s.clone(),
                                _ => val.to_string(),
                            };

                            if let Ok(header_val) = HeaderValue::from_str(&val_str) {
                                headers.insert(header_name, header_val);
                            }
                        }
                    }
                }
            }
            add_json_to_headers(&mut request.headers, jwt.serialized_body);
        }

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
