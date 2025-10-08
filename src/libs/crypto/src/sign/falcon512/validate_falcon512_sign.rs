use oqs::sig::{Algorithm, Sig};

use crate::Error;

pub fn validate_falcon512_sign(
    input: &Vec<u8>,
    signature: &Vec<u8>,
    public_key: &Vec<u8>,
) -> Result<bool, Error> {
    let sig_alg =
        Sig::new(Algorithm::Falcon512).map_err(|e| Error::AlgorithmError(e.to_string()))?;
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
