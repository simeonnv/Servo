use crate::jwt_authorize;
use crate::proxy_ctx::{AfterFilterCTX, ProxyCTX};
use crate::redis_cache::RedisCache;
use crate::server_map::{DownStreamHost, ServerMap};
use async_trait::async_trait;
use bytes::Bytes;
use fred::prelude::KeysInterface;
use http::Uri;
use log::{debug, error, info, warn};
use pingora::ErrorType::HTTPStatus;
use pingora::cache::cache_control::CacheControl;
use pingora::cache::filters::resp_cacheable;
use pingora::cache::{CacheKey, CacheMeta, CacheMetaDefaults, NoCacheReason, RespCacheable};
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::{Error, Result};
use pingora::{
    prelude::HttpPeer,
    proxy::{ProxyHttp, Session},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{Duration, SystemTime};

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
        let downstream_ip = match session.client_addr().map(|e| e.as_inet()) {
            Some(Some(e)) => e.ip(),
            _ => {
                debug!("there is no ip/valid ip in header");
                return Ok(true);
            }
        };

        let req_header = session.req_header_mut();
        let endpoint = req_header.uri.path();

        let host_header = match DownStreamHost::try_from(req_header as &RequestHeader) {
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

        let jwt = if let Some(upstream_auth) = &upstream.auth
            && upstream_auth.jwt_required
        {
            let jwt = jwt_authorize(req_header, upstream_auth).map_err(|err| {
                info!("jwt error: {err}");
                Error::explain(HTTPStatus(401), "Unauthorized")
            })?;

            if let Some(obj) = jwt.serialized_body.as_object() {
                if let Some(required_roles) = &upstream_auth.jwt_auth_roles {
                    let has_access = obj
                        .get("roles")
                        .and_then(|v| v.as_array())
                        .map(|roles| {
                            roles
                                .iter()
                                .filter_map(|r| r.as_str())
                                .any(|r| required_roles.contains(r))
                        })
                        .unwrap_or(false);

                    if !has_access {
                        return Err(Error::explain(
                            HTTPStatus(403),
                            "Forbidden: Missing required roles",
                        ));
                    }
                }

                for (key, val) in obj {
                    let val_str = match val {
                        serde_json::Value::String(s) => s.clone(),
                        _ => val.to_string(),
                    };

                    if let Err(e) = req_header.insert_header(key.to_owned(), &val_str) {
                        warn!("Failed to insert header {key}: {e}");
                    }
                }
            }
            Some(jwt)
        } else {
            None
        };

        let body = session.read_request_body().await?;
        if let Some(data) = body {
            let mut hasher = DefaultHasher::new();
            hasher.write(&data);
            ctx.body_hash = Some(hasher.finish());
        };

        let after_filter_ctx = AfterFilterCTX {
            server: server.clone(),
            host_header,
            upstream,
            jwt,
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
        Ok(())
    }

    fn request_cache_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<()> {
        let ctx_after_filter = ctx.after_filter.as_ref().unwrap();
        if let Some(upstream_cache) = &ctx_after_filter.upstream.cache {
            log::info!("cache enabled");
            session
                .cache
                .enable(upstream_cache.cache, None, None, None, None);
        }

        Ok(())
    }

    fn cache_key_callback(&self, session: &Session, ctx: &mut Self::CTX) -> Result<CacheKey> {
        log::info!("cache key callback");
        let req_header = session.req_header();
        let ctx_after_filter = ctx.after_filter.as_ref().unwrap();

        let host = req_header
            .headers
            .get(http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .or_else(|| req_header.uri.authority().map(|a| a.as_str()))
            .unwrap_or("");

        let path_and_query = req_header
            .uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/");

        let jwt_body = ctx_after_filter
            .jwt
            .as_ref()
            .map(|e| String::from_utf8_lossy(&e.body).into_owned())
            .unwrap_or("None".into());

        let body_hash = match ctx.body_hash {
            Some(ref e) => e.to_string(),
            None => "no_body".into(),
        };

        let key = format!("{host}:{path_and_query}:{jwt_body}:{body_hash}");
        dbg!(&key);
        Ok(CacheKey::new(String::new(), key, String::new()))
    }

    fn response_cache_filter(
        &self,
        _session: &Session,
        resp: &ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<RespCacheable> {
        let ctx_after_filter = ctx.after_filter.as_ref().unwrap();

        if let Some(e) = &ctx_after_filter.upstream.cache {
            let created = SystemTime::now();
            let fresh_until = created + Duration::from_secs(e.cache_time_secs);

            let stale_sec = e.cache_time_secs as u32;

            let cache_meta =
                CacheMeta::new(fresh_until, created, stale_sec, stale_sec, resp.clone());

            Ok(RespCacheable::Cacheable(cache_meta))
        } else {
            Ok(RespCacheable::Uncacheable(NoCacheReason::NeverEnabled))
        }
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
            rest.to_string()
        } else {
            format!("/{}", rest)
        }
    } else {
        path.to_string()
    }
}
