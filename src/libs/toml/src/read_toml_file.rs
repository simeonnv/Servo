use std::fs;

use crate::Error;
use serde::Deserialize;

pub fn read_toml_file<T: for<'a> Deserialize<'a>>(location: &'static str) -> Result<T, Error> {
    let file = fs::read_to_string(location)?;
    let toml: T = toml::from_str(&file)?;
    Ok(toml)
}
