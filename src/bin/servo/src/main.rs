use env_logger::Env;
use log::info;
use pingora::{proxy::http_proxy_service, server::Server};
use servo_toml::{read_or_create_toml, tomls::ConfigToml};

use crate::proxy_state::ProxyState;

mod proxy_state;

mod route_map;
pub use route_map::RouteMap;

fn main() -> Result<(), std::io::Error> {
    let config_toml = read_or_create_toml::<ConfigToml>("./config.toml")
        .unwrap_or_else(|err| panic!("config toml load err => {err}"));
    env_logger::Builder::from_env(Env::default().default_filter_or(&config_toml.config.log_level))
        .init();

    let mut my_server =
        Server::new(None).unwrap_or_else(|err| panic!("Error loading server: {err}"));
    my_server.bootstrap();

    let mut proxy = http_proxy_service(
        &my_server.configuration,
        ProxyState {
            config: config_toml.clone(),
        },
    );

    for addr in &config_toml.config.listens {
        proxy.add_tcp(addr);
        info!("Server binded on: {addr}")
    }

    my_server.add_service(proxy);

    info!("Server starting up!");
    drop(config_toml);
    my_server.run_forever();
}
