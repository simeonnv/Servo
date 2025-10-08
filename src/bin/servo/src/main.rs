use servo_toml::{read_or_create_toml, tomls::ConfigToml};

fn main() {
    let config_toml = read_or_create_toml::<ConfigToml>("./config.toml");
    let config_toml = match config_toml {
        Ok(e) => e,
        Err(e) => panic!("config toml load err: {e}"),
    };

    dbg!(config_toml);

    println!("Hello, world!");
}
