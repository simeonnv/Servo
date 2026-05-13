use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use fred::prelude::{ClientLike, Config, EventInterface, TcpConfig};
use fred::types::Builder;
use log::error;
use matchit::Router;
use thiserror::Error;
use tokio::time::sleep;

use crate::public_pem::Error as PublicPemErr;
use crate::redis_cache::RedisCache;
use crate::server_map::proxy_pass::Error as ProxyPassError;
use crate::server_map::upstream::UpstreamCache;
use crate::server_map::{RateLimiter, Upstream, UpstreamAuth};
use crate::{config_toml::ServerToml, public_pem::PublicPemSync, server_map::ProxyPass};

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub routes: Router<Arc<Upstream>>,
}

impl Server {
    pub async fn from_server_toml(server_toml: &ServerToml) -> Result<Self, Error> {
        let mut router = Router::new();

        let public_key_sync = match &server_toml.auth {
            Some(auth_toml) => {
                let public_pem_sync = loop {
                    let public_pem_sync = PublicPemSync::init_auth_toml(auth_toml).await;
                    let public_pem_sync = match public_pem_sync {
                        Ok(e) => e,
                        Err(PublicPemErr::FailedToFetchPublicPem(err)) => {
                            error!(
                                "failed to fetch public pem {err}, retrying in 10 secs, blocking till successfull"
                            );
                            sleep(Duration::from_secs(10)).await;
                            continue;
                        }
                        Err(err) => panic!("failed to get public pem: {err}"),
                    };
                    break public_pem_sync;
                };

                Some(Arc::new(public_pem_sync))
            }
            None => None,
        };

        let redis_pool = match server_toml.cache {
            Some(ref e) => {
                let config = Config::from_url(e.url.as_str()).expect("invalid cache url");

                let redis_pool = Builder::from_config(config)
                    .with_connection_config(|config| {
                        config.connection_timeout = Duration::from_secs(5);
                        config.tcp = TcpConfig {
                            nodelay: Some(true),
                            ..Default::default()
                        };
                    })
                    .build()
                    .map_err(|e| Error::RedisClient(e.to_string()))?;

                redis_pool
                    .init()
                    .await
                    .map_err(|e| Error::RedisConn(e.to_string()))?;

                redis_pool.on_error(|(error, server)| async move {
                    println!("Redis connection error {:?}: {:?}", server, error);
                    Ok(())
                });

                let cache = Box::leak(Box::new(RedisCache::new(redis_pool))) as &'static RedisCache;

                Some(cache)
            }
            None => None,
        };

        for location_toml in &server_toml.locations {
            let rate_limiter = location_toml
                .max_requests_per_sec
                .map(|e| Arc::new(RateLimiter::new(e as isize)));

            let mut blacklisted_endpoints = HashSet::new();
            for blacklisted_endpoint in location_toml
                .blacklisted_endpoints
                .clone()
                .unwrap_or_default()
            {
                blacklisted_endpoints.insert(blacklisted_endpoint);
            }

            for endpoint in location_toml.endpoints.clone() {
                let proxy_pass = ProxyPass::try_from(location_toml)?;
                let url_concat_suffix = compute_base_endpoint(&endpoint.path);

                let jwt_allowed_roles =
                    location_toml
                        .jwt_allowed_roles
                        .as_ref()
                        .map(|allowed_roles| {
                            let mut jwt_allowed_roles = HashSet::new();
                            for role in allowed_roles {
                                jwt_allowed_roles.insert(role.to_owned());
                            }
                            jwt_allowed_roles
                        });

                let upstream_auth = public_key_sync
                    .as_ref()
                    .map(|public_pem_sync| UpstreamAuth {
                        public_pem_sync: public_pem_sync.clone(),
                        jwt_required: location_toml.requires_jwt.unwrap_or(false),
                        jwt_auth_roles: jwt_allowed_roles,
                    });

                let redis_pool = if location_toml.cacheable.unwrap_or(false) {
                    Some(redis_pool)
                } else {
                    None
                };
                let redis_pool = redis_pool.flatten();

                let rate_limiter = rate_limiter.clone();
                let blacklisted_endpoints = blacklisted_endpoints.clone();
                let upstream_cache = redis_pool.map(|e| UpstreamCache {
                    cache: e,
                    cache_time_secs: location_toml.cache_time_secs.unwrap_or(60 * 60),
                });

                let upstream = Upstream {
                    url_concat_suffix,
                    proxy_pass,
                    rate_limiter,
                    blacklisted_endpoints,
                    auth: upstream_auth,
                    cache: upstream_cache,
                    reroute_template: endpoint.reroute,
                };

                router
                    .insert(endpoint.path, Arc::new(upstream))
                    .map_err(|err| Error::FailedToInsertIntoRouter(err.to_string()))?;
            }
        }

        let server = Server {
            name: server_toml.name.clone(),
            routes: router,
        };

        Ok(server)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to build proxypass => {0}")]
    FailedToBuildProxyPass(#[from] ProxyPassError),

    #[error("Failed to insert into router => {0}")]
    FailedToInsertIntoRouter(String),

    #[error("Failed to create a redis client => {0}")]
    RedisClient(String),

    #[error("Failed to get redis connection {0}")]
    RedisConn(String),
}

/// Extracts the static base portion of a URL pattern string.
///
/// This function iterates through a path separated by forward slashes (`/`)
/// and collects segments until it encounters a dynamic parameter (indicated
/// by a segment starting with `{`).
///
/// ### Behavior:
/// * Returns `/` if the pattern is empty or starts immediately with a parameter.
/// * Trims empty segments (e.g., double slashes `//`).
/// * Joins the static segments into a single path string prefixed with `/`.
///
/// # Arguments
/// * `pattern` - A string slice representing the URL path (e.g., "api/v1/users/{id}").
///
/// # Examples
/// ```
/// let base = compute_base_endpoint("api/v1/users/{id}");
/// assert_eq!(base, "/api/v1/users");
///
/// let root = compute_base_endpoint("{id}");
/// assert_eq!(root, "/");
/// ```
fn compute_base_endpoint(pattern: &str) -> String {
    let parts: Vec<&str> = pattern.split('/').collect();
    let mut base_parts = Vec::new();

    for part in parts {
        if part.starts_with('{') {
            break;
        }
        if !part.is_empty() {
            base_parts.push(part);
        }
    }

    if base_parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", base_parts.join("/"))
    }
}
