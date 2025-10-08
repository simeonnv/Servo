use std::fs;

use crate::Error;
use serde::Serialize;

pub fn create_toml_file<T: Serialize>(input: &T, location: &'static str) -> Result<(), Error> {
    let toml_string = toml::to_string(input)?;
    fs::File::create(location)?;
    fs::write(location, toml_string)?;
    Ok(())
}
