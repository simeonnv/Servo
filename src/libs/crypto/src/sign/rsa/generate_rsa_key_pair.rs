use chrono::Utc;
use openssl::rsa::Rsa;

use crate::{Error, sign::key_pair::KeyPair};

pub fn generate_rsa_key_pair() -> Result<KeyPair, Error> {
    let rsa = Rsa::generate(2048).map_err(|e| Error::KeyGenerateError(e.to_string()))?;

    let private_key = rsa
        .private_key_to_pem()
        .map_err(|e| Error::KeyGenerateError(e.to_string()))?;
    let public_key = rsa
        .public_key_to_pem()
        .map_err(|e| Error::KeyGenerateError(e.to_string()))?;

    let now = Utc::now().naive_utc();

    Ok(KeyPair {
        private_key,
        public_key,
        creation_time: now,
    })
}
