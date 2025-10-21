use std::collections::HashSet;

use matchit::Router;
use serde::ser;
use thiserror::Error;

use crate::server_map::proxy_pass::Error as ProxyPassError;
use crate::{config_toml::ServerToml, public_pem::PublicPemSync, server_map::ProxyPass};

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub routes: Router<ProxyPass>,
    pub public_key_sync: Option<PublicPemSync>,
}

impl Server {
    pub async fn from_server_toml(server_toml: &ServerToml) -> Result<Self, Error> {
        let mut router = Router::new();

        for location_toml in &server_toml.locations {
            for endpoint in location_toml.endpoints.clone() {
                let proxy_pass = ProxyPass::try_from(location_toml)?;
                router
                    .insert(endpoint, proxy_pass)
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
