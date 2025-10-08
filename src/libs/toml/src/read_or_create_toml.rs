use log::warn;
use serde::{Deserialize, Serialize};

use crate::{Error, create_toml_file, read_toml_file};

pub fn read_or_create_toml<T: Serialize + for<'a> Deserialize<'a> + Default>(
    location: &'static str,
) -> Result<T, Error> {
    let toml = read_toml_file::<T>(location);
    match toml {
        Ok(e) => Ok(e),
        Err(Error::FileSystemError(err)) => {
            warn!("failed to read toml at: {location}, with: {err}, initing default",);
            let default_toml = T::default();
            create_toml_file(&default_toml, location)?;
            Ok(default_toml)
        }
        Err(err) => Err(err),
    }
}
