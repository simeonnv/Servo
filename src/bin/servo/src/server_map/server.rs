use std::sync::Arc;

use matchit::Router;
use thiserror::Error;

use crate::server_map::proxy_pass::Error as ProxyPassError;
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

        let public_key_sync = if let Some(auth_toml) = &server_toml.auth {
            Some(
                PublicPemSync::init_auth_toml(auth_toml)
                    .await
                    .unwrap_or_else(|err| {
                        panic!("Failed to build public pem synchronizer => {err}");
                    }),
            )
        } else {
            None
        };
        let public_key_sync = public_key_sync.map(|e| Arc::new(e));

        for location_toml in &server_toml.locations {
            let rate_limiter = location_toml
                .max_requests_per_sec
                .map(|e| Arc::new(RateLimiter::new(e as isize)));

            for endpoint in location_toml.endpoints.clone() {
                let proxy_pass = ProxyPass::try_from(location_toml)?;
                let url_concat_suffix = compute_base_endpoint(&endpoint);

                let upstream_auth = public_key_sync
                    .as_ref()
                    .map(|public_pem_sync| UpstreamAuth {
                        public_pem_sync: public_pem_sync.clone(),
                        jwt_required: location_toml.requires_jwt.unwrap_or(false),
                        jwt_auth_roles: location_toml.jwt_allowed_roles.clone(),
                    });

                let rate_limiter = rate_limiter.clone();
                let upstream = Upstream {
                    url_concat_suffix,
                    proxy_pass,
                    rate_limiter,
                    auth: upstream_auth,
                };

                router
                    .insert(endpoint, Arc::new(upstream))
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
}

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
