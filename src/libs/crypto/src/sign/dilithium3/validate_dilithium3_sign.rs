use crate::Error;
use oqs::sig::{Algorithm, Sig};

pub fn validate_dilithium3_sign(
    input: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Result<bool, Error> {
    let sig_alg =
        Sig::new(Algorithm::Dilithium3).map_err(|e| Error::AlgorithmError(e.to_string()))?;

    let public_key = sig_alg
        .public_key_from_bytes(public_key)
        .ok_or(Error::InvalidKeyError("".into()))?;
    let signature = sig_alg.signature_from_bytes(&signature);
    let signature = match signature {
        Some(e) => e,
        None => return Ok(false),
    };

    let verified = sig_alg.verify(input, signature, public_key).is_ok();

    Ok(verified)
}
