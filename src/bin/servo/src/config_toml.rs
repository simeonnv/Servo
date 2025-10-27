use std::{collections::HashSet, net::SocketAddr, path::PathBuf};

use log::Level;
use serde::{Deserialize, Serialize};
use servo_toml::FormatValidate;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigToml {
    pub config: GatewayConfigToml,
    pub servers: Vec<ServerToml>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GatewayConfigToml {
    pub gateway_name: String,
    pub listens: Vec<SocketAddr>,
    pub log_level: Level,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerToml {
    pub name: String,
    pub downstream_hosts: Vec<String>,
    pub auth: Option<AuthToml>,
    pub locations: Vec<LocationToml>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationToml {
    pub endpoints: Vec<String>,
    // pub blacklisted_endpoints: Vec<String>,
    pub max_requests_per_sec: Option<usize>,
    pub proxy_passes: Vec<SocketAddr>,
    pub health_check: Option<bool>,
    pub health_check_frequency: Option<u64>,
    pub requires_jwt: Option<bool>,
    pub jwt_allowed_roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthToml {
    #[serde(flatten)]
    pub public_pem_location: PublicPemLocationToml,
    pub check_duration: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PublicPemLocationToml {
    PublicPemHttpUrl(Url),
    PublicPemPath(PathBuf),
}

impl Default for ConfigToml {
    fn default() -> Self {
        let server_1 = ServerToml {
            name: "test".into(),
            downstream_hosts: vec!["someaddress.com".into(), "0.0.0.0:54321".into()],
            auth: Some(AuthToml {
                public_pem_location: PublicPemLocationToml::PublicPemHttpUrl(
                    Url::parse("http://0.0.0.0:25025/public_pem").unwrap(),
                ),
                check_duration: 10_000,
            }),
            locations: vec![LocationToml {
                endpoints: vec!["/".into(), "/{*any}".into()],
                health_check: Some(true),
                health_check_frequency: Some(3000),
                proxy_passes: vec!["192.168.1.103:8080".parse().unwrap()],
                max_requests_per_sec: Some(10),
                requires_jwt: Some(true),
                jwt_allowed_roles: Some(vec!["user".into()]),
            }],
        };

        let config = GatewayConfigToml {
            gateway_name: "give me a name vro".into(),
            listens: vec!["0.0.0.0:54321".parse().unwrap()],
            log_level: Level::Info,
        };

        Self {
            config,
            servers: vec![server_1],
        }
    }
}

impl FormatValidate for ConfigToml {
    fn validate(&self) -> Result<(), String> {
        if has_duplicates(&self.config.listens) {
            return Err("Duplicate proxy listen addresses!".into());
        }

        let upstream_names: Vec<String> = self.servers.iter().map(|e| e.name.clone()).collect();
        if has_duplicates(&upstream_names) {
            return Err("2 or more servers have the same name!".into());
        }

        let all_downstream_hosts: Vec<String> = self
            .servers
            .iter()
            .flat_map(|upstream| upstream.downstream_hosts.clone())
            .collect();
        if has_duplicates(&all_downstream_hosts) {
            return Err("Duplicate downstream hosts found across servers!".into());
        }

        let all_location_endpoint_patterns: Vec<String> = self
            .servers
            .iter()
            .flat_map(|upstream| upstream.locations.iter())
            .flat_map(|location| location.endpoints.iter())
            .cloned()
            .collect();
        if !all_location_endpoint_patterns
            .iter()
            .all(|endpoint| endpoint.starts_with('/'))
        {
            return Err("Not all endpoint patterns start with '/'!".into());
        }

        for upstream in &self.servers {
            let endpoints: Vec<String> = upstream
                .locations
                .iter()
                .flat_map(|loc| loc.endpoints.iter())
                .cloned()
                .collect();
            if has_duplicates(&endpoints) {
                return Err(format!(
                    "Duplicate endpoint patterns found in upstream '{}'",
                    upstream.name
                ));
            }
        }

        Ok(())
    }
}

fn has_duplicates<T: Eq + std::hash::Hash>(vec: &[T]) -> bool {
    let mut seen = HashSet::new();
    for item in vec {
        if !seen.insert(item) {
            return true; // Duplicate found
        }
    }
    false // No duplicates
}
