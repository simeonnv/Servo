use chrono::Utc;
use oqs::kem::{Algorithm, Kem};

use crate::{Error, sign::key_pair::KeyPair};

pub fn generate_ml_kem1024_key_pair() -> Result<KeyPair, Error> {
    let kem_alg =
        Kem::new(Algorithm::MlKem1024).map_err(|e| Error::AlgorithmError(e.to_string()))?;
    let (public_key, private_key) = kem_alg
        .keypair()
        .map_err(|e| Error::KeyGenerateError(e.to_string()))?;
    let now = Utc::now().naive_utc();
    // kem_alg.
    let keypair = KeyPair {
        public_key: public_key.into_vec(),
        private_key: private_key.into_vec(),
        creation_time: now,
    };

    Ok(keypair)
}
