use env_logger::Env;
use log::{info, warn};
use openssl::ssl::{SslAlert, SslRef};
use pingora::{listeners::tls::TlsSettings, proxy::http_proxy_service, server::Server};
use servo_toml::read_or_create_toml;
use tokio::runtime::Runtime;

use crate::{proxy::Proxy, tls::CertificateConfig};

mod proxy;

pub mod proxy_ctx;

mod jwt_authorize;
pub use jwt_authorize::jwt_authorize;

pub mod server_map;
use server_map::ServerMap;

mod config_toml;
pub use config_toml::ConfigToml;

pub mod public_pem;

pub mod tls;

fn main() -> Result<(), std::io::Error> {
    let config_toml = read_or_create_toml::<ConfigToml>("./Config.toml")
        .unwrap_or_else(|err| panic!("config toml load err => {err}"));
    env_logger::Builder::from_env(
        Env::default().default_filter_or(config_toml.config.log_level.as_str()),
    )
    .init();

    let mut my_server =
        Server::new(None).unwrap_or_else(|err| panic!("Error loading server: {err}"));
    my_server.bootstrap();

    let rt = Runtime::new().unwrap();
    let server_map = rt.block_on(ServerMap::build_from_config_toml(&config_toml));
    let mut proxy = http_proxy_service(&my_server.configuration, Proxy { server_map });

    for addr in &config_toml.config.listens {
        proxy.add_tcp(&addr.to_string());
        info!("Server binded on: {addr}")
    }

    if let Some(ref e) = config_toml.config.tls {
        let mut certificate_configs: Vec<tls::CertificateConfig> = Vec::new();
        for tls_toml in e {
            certificate_configs.push(CertificateConfig {
                cert_path: tls_toml.cert_path.to_str().unwrap().to_owned(),
                key_path: tls_toml.key_path.to_str().unwrap().to_owned(),
            });
        }
        let certificates = tls::Certificates::new(&certificate_configs);
        let mut tls_settings = TlsSettings::intermediate(
            &certificates.default_cert_path,
            &certificates.default_key_path,
        )
        .expect("unable to load or parse cert/key");
        tls_settings.set_servername_callback(
            move |ssl_ref: &mut SslRef, ssl_alert: &mut SslAlert| {
                certificates.server_name_callback(ssl_ref, ssl_alert)
            },
        );
        tls_settings.enable_h2();
        tls_settings.set_alpn_select_callback(tls::prefer_h2);
        warn!("binding on 0.0.0.0:443, bc tls is supplied");
        proxy.add_tls_with_settings("0.0.0.0:443", None, tls_settings);
    }

    my_server.add_service(proxy);

    info!("Server starting up!");
    drop(config_toml);
    my_server.run_forever();
}
