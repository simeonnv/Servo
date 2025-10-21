use envconfig::Envconfig;
use lazy_static::lazy_static;

use crate::env::load_env_vars;

#[derive(Envconfig)]
pub struct EnvVars {
    #[envconfig(from = "RUST_LOG")]
    pub rust_log: String,

    #[envconfig(from = "AUTH_ADDRESS")]
    pub auth_address: String,

    #[envconfig(from = "AUTH_PORT", default = "8989")]
    pub auth_port: u32,

    #[envconfig(from = "POSTGRES_NAME")]
    pub postgres_name: String,

    #[envconfig(from = "POSTGRES_USER")]
    pub postgres_user: String,

    #[envconfig(from = "POSTGRES_PASSWORD")]
    pub postgres_password: String,

    #[envconfig(from = "DB_ADDRESS")]
    pub db_address: String,

    #[envconfig(from = "DB_PORT", default = "5432")]
    pub db_port: u16,

    #[envconfig(from = "POOL_MAX_CONN", default = "5")]
    pub pool_max_conn: u32,
}

lazy_static! {
    pub static ref ENVVARS: EnvVars = load_env_vars();
}
