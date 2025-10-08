use crate::Error;
use aes_gcm::{
    Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use oqs::kem::{Algorithm, Kem};

pub fn encrypt_ml_kem1024(
    input: &Vec<u8>,
    public_key: &Vec<u8>,
) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let kem_alg =
        Kem::new(Algorithm::MlKem1024).map_err(|e| Error::AlgorithmError(e.to_string()))?;
    let public_key = kem_alg
        .public_key_from_bytes(public_key)
        .ok_or(Error::InvalidKeyError(
            "invalid MLKEM1034 public key!".into(),
        ))?;

    let (kem_ciphertext, shared_secret) = kem_alg
        .encapsulate(public_key)
        .map_err(|e| Error::EncryptionError(e.to_string()))?;

    let key = Key::<Aes256Gcm>::from_slice(shared_secret.as_ref());
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, input.as_slice())
        .map_err(|e| Error::EncryptionError(e.to_string()))?;

    let mut full_ciphertext = nonce_bytes.to_vec();
    full_ciphertext.extend(ciphertext);

    Ok((kem_ciphertext.into_vec(), full_ciphertext))
}
