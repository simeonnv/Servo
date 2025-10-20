use std::time::Duration;

use actix_web::{App, HttpServer, web::Data};
use env_logger::Env;

pub mod routes;
use key_pair_roller::KeyPairRoller;
use routes::get_ping::get_ping;

use crate::routes::get_public_pem::get_public_pem;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let key_pair_roller = KeyPairRoller::init_rsa_roller(Duration::from_secs(60)).unwrap();
    let key_pair_roller = Data::new(key_pair_roller);

    HttpServer::new(move || {
        App::new()
            .app_data(key_pair_roller.clone())
            .service(get_ping)
            .service(get_public_pem)
    })
    .bind(("0.0.0.0", 8989))?
    .run()
    .await
}
