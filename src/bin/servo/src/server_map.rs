use std::sync::Arc;

use dashmap::DashMap;
use servo_toml::tomls::config_toml::ConfigToml;
use servo_types::{DownStreamHost, Server};

#[derive(Debug)]
pub struct ServerMap {
    // a router hashmap that contains the key: downstream host
    // and val: endpoint router with the proxy pass for that downstream
    pub routes: DashMap<DownStreamHost<'static>, Arc<Server>>,
}

// peak code
impl ServerMap {
    pub fn build_from_config_toml(config: &ConfigToml) -> Self {
        let routes = DashMap::new();
        for server_toml in &config.servers {
            let server = match Server::try_from(server_toml) {
                Ok(server) => Arc::new(server),
                Err(e) => {
                    eprintln!("Failed to create server from config: {}", e);
                    continue;
                }
            };
            for downstream_host_toml in &server_toml.downstream_hosts {
                routes.insert(
                    DownStreamHost::from(downstream_host_toml.to_owned()),
                    server.clone(),
                );
            }
        }
        Self { routes }
    }
}
