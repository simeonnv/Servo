use actix_web::{App, HttpServer, web::Data};
use env_logger::Env;

pub mod env;

pub mod routes;
use key_pair_roller::KeyPairRoller;

mod create_postgres_pool;
pub use create_postgres_pool::create_postgres_pool;

mod error;
pub use error::Error;

pub mod config;

pub mod api_docs;

use crate::{config::KEY_PAIR_LIFETIME, env::ENVVARS, routes::routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let _ = ENVVARS.rust_log;

    let key_pair_roller =
        KeyPairRoller::init_rsa_roller(KEY_PAIR_LIFETIME.to_std().unwrap()).unwrap();
    let key_pair_roller = Data::new(key_pair_roller);
    let db_pool = create_postgres_pool(
        &ENVVARS.postgres_user,
        &ENVVARS.postgres_password,
        &ENVVARS.db_address,
        ENVVARS.db_port,
        &ENVVARS.postgres_name,
        ENVVARS.pool_max_conn,
    )
    .await
    .unwrap();
    let db_pool = Data::new(db_pool);

    HttpServer::new(move || {
        App::new()
            .app_data(key_pair_roller.clone())
            .app_data(db_pool.clone())
            .service(routes())
    })
    .bind(("0.0.0.0", 8989))?
    .run()
    .await
}
