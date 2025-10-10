use std::{rc::Rc, sync::Arc};

use dashmap::DashMap;
use matchit::Router;
use servo_toml::tomls::ConfigToml;

#[derive(Debug)]
pub struct ServerMap {
    // a router hashmap that contains the key: downstream host
    // and val: endpoint router with the proxy pass for that downstream
    pub routes: DashMap<DownStreamHost, Arc<Server>>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct DownStreamHost(pub String);

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub routes: Router<ProxyPass>,
}

#[derive(Debug)]
pub struct ProxyPass(pub String);

impl ServerMap {
    pub fn build_from_config_toml(config: &ConfigToml) -> Self {
        let routes = DashMap::new();

        for server_toml in &config.servers {
            let mut router = Router::new();
            for location_toml in &server_toml.locations {
                router
                    .insert(
                        location_toml.endpoint.clone(),
                        ProxyPass(location_toml.proxy_pass.clone()),
                    )
                    .expect("invalid location endpoint!");
            }
            let router = router;
            let server = Arc::new(Server {
                name: server_toml.name.clone(),
                routes: router,
            });
            for downstream_host_toml in &server_toml.downstream_hosts {
                routes.insert(DownStreamHost(downstream_host_toml.clone()), server.clone());
            }
        }

        Self { routes }
    }
}
