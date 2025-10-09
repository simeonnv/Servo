use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to serialize toml => {0}")]
    SerializationError(#[from] toml::ser::Error),

    #[error("Unable to deserialize toml => {0}")]
    DeserializationError(#[from] toml::de::Error),

    #[error("There is a error in the toml logic => {0}")]
    TomlValidationError(String),

    #[error("Unable to create file / read file => {0}")]
    FileSystemError(#[from] std::io::Error),
}
