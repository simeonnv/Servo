use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce, aead::Aead};
use oqs::kem::{Algorithm, Kem};

use crate::Error;

pub fn decrypt_ml_kem1024(
    ciphertext: &Vec<u8>,
    kem_ciphertext: &Vec<u8>,
    private_key: &Vec<u8>,
) -> Result<Vec<u8>, Error> {
    let kem_alg =
        Kem::new(Algorithm::MlKem1024).map_err(|e| Error::AlgorithmError(e.to_string()))?;

    let private_key = kem_alg
        .secret_key_from_bytes(private_key)
        .ok_or(Error::InvalidKeyError("".into()))?;

    let kem_ciphertext =
        kem_alg
            .ciphertext_from_bytes(kem_ciphertext)
            .ok_or(Error::InvalidCipthertextError(
                "the ciptertext doesent corespond to MLKEM1024".into(),
            ))?;

    let shared_secret = kem_alg
        .decapsulate(private_key, kem_ciphertext)
        .map_err(|e| Error::DecryptionError(e.to_string()))?;

    let key = Key::<Aes256Gcm>::from_slice(shared_secret.as_ref());
    let cipher = Aes256Gcm::new(key);

    if ciphertext.len() < 12 {
        return Err(Error::InvalidCipthertextError(
            "Invalid symmetric ciphertext".into(),
        ));
    }

    let nonce = Nonce::from_slice(&ciphertext[0..12]);
    let ciphertext = &ciphertext[12..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| Error::DecryptionError(e.to_string()))?;

    Ok(plaintext)
}
