use matchit::Router;
use servo_toml::tomls::config_toml::ServerToml;
use thiserror::Error;

use crate::ProxyPass;
use crate::proxy_pass::Error as ProxyPassError;

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub routes: Router<ProxyPass>,
}

impl TryFrom<&ServerToml> for Server {
    type Error = Error;

    fn try_from(server_toml: &ServerToml) -> Result<Self, Error> {
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
