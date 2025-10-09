use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::FormatValidate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigToml {
    pub config: Config,
    pub upstreams: Vec<Upstream>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub gateway_name: String,
    pub listens: Vec<String>,
    pub log_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upstream {
    pub name: String,
    pub downstream_addresses: Vec<String>,
    pub location: Vec<Location>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    endpoint: String,
    proxy_pass: String,
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

        let upstream_names: Vec<String> = self.upstreams.iter().map(|e| e.name.clone()).collect();
        if has_duplicates(&upstream_names) {
            return Err("2 or more upstreams have the same name!".into());
        }

        let all_downstream_addresses: Vec<String> = self
            .upstreams
            .iter()
            .flat_map(|upstream| upstream.downstream_addresses.clone())
            .collect();
        if has_duplicates(&all_downstream_addresses) {
            return Err("Duplicate downstream addresses found across upstreams!".into());
        }

        let all_location_endpoint_patterns: Vec<String> = self
            .upstreams
            .iter()
            .flat_map(|upstream| upstream.location.clone())
            .map(|e| e.endpoint)
            .collect();
        if !all_location_endpoint_patterns
            .iter()
            .all(|endpoint| endpoint.starts_with('/'))
        {
            return Err("Not all endpoint patterns start with '/'!".into());
        }

        // New check for duplicate endpoints within each upstream
        for upstream in &self.upstreams {
            let endpoints: Vec<String> = upstream
                .location
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
        let upstream_1 = Upstream {
            name: "test".into(),
            downstream_addresses: vec!["someaddress.com".into()],
            location: vec![Location {
                endpoint: "/".into(),
                proxy_pass: "http://192.168.1.103:8080".into(),
            }],
        };

        let config = Config {
            gateway_name: "give me a name vro".into(),
            listens: vec!["0.0.0.0:54321".into()],
            log_level: "info".into(),
        };

        Self {
            config,
            upstreams: vec![upstream_1],
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
