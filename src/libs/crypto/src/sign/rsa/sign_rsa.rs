use crate::Error;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Signer};

pub fn sign_rsa(input: &[u8], private_key: &[u8]) -> Result<Vec<u8>, Error> {
    let private_key = PKey::private_key_from_pem(private_key)
        .map_err(|e| Error::InvalidKeyError(e.to_string()))?;

    let mut signer = Signer::new(MessageDigest::sha256(), &private_key)
        .map_err(|e| Error::AlgorithmError(e.to_string()))?;
    signer
        .update(input)
        .map_err(|e| Error::EncryptionError(e.to_string()))?;
    let signature = signer
        .sign_to_vec()
        .map_err(|e| Error::EncryptionError(e.to_string()))?;

    Ok(signature)
}
