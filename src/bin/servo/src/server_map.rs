use core::fmt;
use std::net::SocketAddr;
use std::time::Duration;
use std::{borrow::Borrow, sync::Arc};

use dashmap::DashMap;
use matchit::Router;
use pingora::lb::LoadBalancer;
use pingora::prelude::{RoundRobin, TcpHealthCheck, background_service};
use servo_toml::tomls::ConfigToml;
use url::Host;

#[derive(Debug)]
pub struct ServerMap {
    // a router hashmap that contains the key: downstream host
    // and val: endpoint router with the proxy pass for that downstream
    pub routes: DashMap<DownStreamHost, Arc<Server>>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct DownStreamHost(pub Host);

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub routes: Router<ProxyPasses>,
}

#[derive(Clone)]
pub struct ProxyPasses {
    pub proxy_passes: Vec<SocketAddr>,
    pub load_balancer: Arc<LoadBalancer<RoundRobin>>,
}

impl fmt::Debug for ProxyPasses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProxyPasses")
            .field("proxy_passes", &self.proxy_passes)
            .field("load_balancer", &"<LoadBalancer<RoundRobin>>")
            .finish()
    }
}

// peak code
impl ServerMap {
    pub fn build_from_config_toml(config: &ConfigToml) -> Self {
        let routes = DashMap::new();

        for server_toml in &config.servers {
            let mut router = Router::new();
            for location_toml in &server_toml.locations {
                for endpoint in location_toml.endpoints.clone() {
                    let mut proxy_passes_loadbalancer: LoadBalancer<RoundRobin> =
                        LoadBalancer::try_from_iter(&location_toml.proxy_passes).unwrap();

                    let proxy_passes_loadbalancer = if location_toml.health_check.unwrap_or(false) {
                        let hc = TcpHealthCheck::new();
                        proxy_passes_loadbalancer.set_health_check(hc);
                        proxy_passes_loadbalancer.health_check_frequency =
                            Some(Duration::from_millis(
                                location_toml.health_check_frequency.unwrap_or(3000),
                            ));
                        let background = background_service(
                            &format!("health check: {}, {}", server_toml.name, endpoint),
                            proxy_passes_loadbalancer,
                        );
                        background.task()
                    } else {
                        Arc::new(proxy_passes_loadbalancer)
                    };

                    let proxy_passes = ProxyPasses {
                        proxy_passes: location_toml.proxy_passes.clone(),
                        load_balancer: proxy_passes_loadbalancer,
                    };

                    router
                        .insert(endpoint, proxy_passes)
                        .expect("invalid location endpoint!");
                }
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
