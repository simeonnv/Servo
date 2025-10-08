use oqs::sig::{Algorithm, Sig};

use crate::Error;

pub fn sign_falcon512(input: &[u8], private_key: &[u8]) -> Result<Vec<u8>, Error> {
    let sig_alg =
        Sig::new(Algorithm::Falcon512).map_err(|e| Error::AlgorithmError(e.to_string()))?;
    let private_key = sig_alg
        .secret_key_from_bytes(private_key)
        .ok_or(Error::InvalidKeyError("".into()))?;

    let signature = sig_alg
        .sign(input, private_key)
        .map_err(|e| Error::EncryptionError(e.to_string()))?;

    let signature = signature.into_vec();

    Ok(signature)
}
