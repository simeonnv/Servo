use env_logger::Env;
use log::info;
use pingora::{proxy::http_proxy_service, server::Server};
use servo_toml::{read_or_create_toml, tomls::ConfigToml};

use crate::proxy_state::ProxyState;

mod proxy_state;

pub mod proxy_ctx;

pub mod server_map;
use server_map::ServerMap;

fn main() -> Result<(), std::io::Error> {
    let config_toml = read_or_create_toml::<ConfigToml>("./config.toml")
        .unwrap_or_else(|err| panic!("config toml load err => {err}"));
    env_logger::Builder::from_env(Env::default().default_filter_or(&config_toml.config.log_level))
        .init();

    let mut my_server =
        Server::new(None).unwrap_or_else(|err| panic!("Error loading server: {err}"));
    my_server.bootstrap();

    let server_map = ServerMap::build_from_config_toml(&config_toml);
    dbg!(&server_map);

    let mut proxy = http_proxy_service(&my_server.configuration, ProxyState { server_map });

    for addr in &config_toml.config.listens {
        proxy.add_tcp(addr);
        info!("Server binded on: {addr}")
    }

    my_server.add_service(proxy);

    info!("Server starting up!");
    drop(config_toml);
    my_server.run_forever();
}
