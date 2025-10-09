use dashmap::DashMap;
use matchit::Router;
use servo_toml::tomls::ConfigToml;

pub struct RouteMap {
    // a router hashmap that contains the key: downstream host
    // and val: endpoint router with the proxy pass for that downstream
    pub routes: DashMap<String, Router<String>>,
}

impl RouteMap {
    pub fn build_from_config(config: &ConfigToml) -> Self {
        let routes = DashMap::new();

        Self { routes }
    }
}
