use matchit::Router;
use thiserror::Error;

use crate::server_map::proxy_pass::Error as ProxyPassError;
use crate::{config_toml::ServerToml, public_pem::PublicPemSync, server_map::ProxyPass};

#[derive(Debug)]
pub struct Server {
    pub name: String,
    // the string is the url concat suffix
    pub routes: Router<(ProxyPass, String)>,
    pub public_key_sync: Option<PublicPemSync>,
}

impl Server {
    pub async fn from_server_toml(server_toml: &ServerToml) -> Result<Self, Error> {
        let mut router = Router::new();

        for location_toml in &server_toml.locations {
            for endpoint in location_toml.endpoints.clone() {
                let proxy_pass = ProxyPass::try_from(location_toml)?;
                let url_concat_suffix = compute_base_endpoint(&endpoint);
                router
                    .insert(endpoint, (proxy_pass, url_concat_suffix))
                    .map_err(|err| Error::FailedToInsertIntoRouter(err.to_string()))?;
            }
        }
        let router = router;

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

        let server = Server {
            name: server_toml.name.clone(),
            public_key_sync,
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
