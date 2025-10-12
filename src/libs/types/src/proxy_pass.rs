use core::fmt;
use pingora_core::prelude::background_service;
use pingora_load_balancing::{
    LoadBalancer,
    prelude::{RoundRobin, TcpHealthCheck},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use thiserror::Error;

use servo_toml::tomls::config_toml::LocationToml;

#[derive(Clone)]
pub struct ProxyPass {
    pub addrs: Vec<SocketAddr>,
    pub load_balancer: Arc<LoadBalancer<RoundRobin>>,
}

impl TryFrom<&LocationToml> for ProxyPass {
    type Error = Error;

    fn try_from(location_toml: &LocationToml) -> Result<Self, Error> {
        let mut proxy_passes_loadbalancer: LoadBalancer<RoundRobin> =
            LoadBalancer::try_from_iter(&location_toml.proxy_passes)
                .map_err(|err| Error::FailedToBuildLoadbalancer(err.to_string()))?;

        let proxy_passes_loadbalancer = if location_toml.health_check.unwrap_or(false) {
            let hc = TcpHealthCheck::new();
            proxy_passes_loadbalancer.set_health_check(hc);
            proxy_passes_loadbalancer.health_check_frequency = Some(Duration::from_millis(
                location_toml.health_check_frequency.unwrap_or(3000),
            ));
            let background = background_service(
                &format!("health check: {}", location_toml.endpoints.join(", ")),
                proxy_passes_loadbalancer,
            );
            background.task()
        } else {
            Arc::new(proxy_passes_loadbalancer)
        };

        let proxy_pass = ProxyPass {
            addrs: location_toml.proxy_passes.clone(),
            load_balancer: proxy_passes_loadbalancer,
        };

        Ok(proxy_pass)
    }
}

impl fmt::Debug for ProxyPass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProxyPasses")
            .field("addrs", &self.addrs)
            .field("load_balancer", &"<LoadBalancer<RoundRobin>>")
            .finish()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to build a loadbalancer => {0}")]
    FailedToBuildLoadbalancer(String),
}
