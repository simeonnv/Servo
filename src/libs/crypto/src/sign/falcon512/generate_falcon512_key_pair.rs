use chrono::Utc;
use oqs::sig::{self, Sig};

use crate::{Error, sign::key_pair::KeyPair};

pub fn generate_falcon512_key_pair() -> Result<KeyPair, Error> {
    let now = Utc::now().naive_utc();
    let sig_alg =
        Sig::new(sig::Algorithm::Falcon512).map_err(|e| Error::KeyGenerateError(e.to_string()))?;
    let (public_key, private_key) = sig_alg
        .keypair()
        .map_err(|e| Error::KeyGenerateError(e.to_string()))?;

    let (public_key, private_key) = (public_key.into_vec(), private_key.into_vec());

    Ok(KeyPair {
        private_key,
        public_key,
        creation_time: now,
    })
}
