use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::FormatValidate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigToml {
    pub config: GatewayConfigToml,
    pub servers: Vec<ServerToml>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GatewayConfigToml {
    pub gateway_name: String,
    pub listens: Vec<String>,
    pub log_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerToml {
    pub name: String,
    pub downstream_hosts: Vec<String>,
    pub locations: Vec<LocationToml>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationToml {
    pub endpoint: String,
    pub proxy_pass: String,
}

impl FormatValidate for ConfigToml {
    fn validate(&self) -> Result<(), String> {
        // Existing checks
        if has_duplicates(&self.config.listens) {
            return Err("Duplicate proxy listen addresses!".into());
        }

        match self.config.log_level.as_str() {
            "debug" | "info" | "warn" | "error" | "fatal" => {}
            _ => return Err("Invalid log level!".into()),
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
            .flat_map(|upstream| upstream.locations.clone())
            .map(|e| e.endpoint)
            .collect();
        if !all_location_endpoint_patterns
            .iter()
            .all(|endpoint| endpoint.starts_with('/'))
        {
            return Err("Not all endpoint patterns start with '/'!".into());
        }

        // New check for duplicate endpoints within each upstream
        for upstream in &self.servers {
            let endpoints: Vec<String> = upstream
                .locations
                .iter()
                .map(|loc| loc.endpoint.clone())
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

impl Default for ConfigToml {
    fn default() -> Self {
        let server_1 = ServerToml {
            name: "test".into(),
            downstream_hosts: vec!["someaddress.com".into()],
            locations: vec![LocationToml {
                endpoint: "/".into(),
                proxy_pass: "http://192.168.1.103:8080".into(),
            }],
        };

        let config = GatewayConfigToml {
            gateway_name: "give me a name vro".into(),
            listens: vec!["0.0.0.0:54321".into()],
            log_level: "info".into(),
        };

        Self {
            config,
            servers: vec![server_1],
        }
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
