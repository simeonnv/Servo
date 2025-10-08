use crate::Error;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Verifier};

pub fn validate_rsa_sign(input: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, Error> {
    let public_key =
        PKey::public_key_from_pem(public_key).map_err(|e| Error::InvalidKeyError(e.to_string()))?;

    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)
        .map_err(|e| Error::AlgorithmError(e.to_string()))?;

    verifier
        .update(input)
        .map_err(|e| Error::DecryptionError(e.to_string()))?;
    let verified = verifier
        .verify(signature)
        .map_err(|e| Error::DecryptionError(e.to_string()))?;

    Ok(verified)
}
