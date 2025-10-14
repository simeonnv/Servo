use std::sync::Arc;

use dashmap::DashMap;

use crate::{
    ConfigToml,
    server_map::{DownStreamHost, Server},
};

#[derive(Debug)]
pub struct ServerMap {
    // a router hashmap that contains the key: downstream host
    // and val: endpoint router with the proxy pass for that downstream
    pub routes: DashMap<DownStreamHost, Arc<Server>>,
}

// peak code
impl ServerMap {
    pub async fn build_from_config_toml(config: &ConfigToml) -> Self {
        let routes = DashMap::new();
        for server_toml in &config.servers {
            let server = match Server::from_server_toml(server_toml).await {
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
