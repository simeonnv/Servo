use argon2::{
    Algorithm, Argon2, Version,
    password_hash::{PasswordHash, PasswordVerifier},
};

use crate::{ARGON2_PARAMS, Error};

pub async fn compare_argon2_hash(input: &String, hash: &String) -> Result<bool, Error> {
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, ARGON2_PARAMS);

    let parsed_hash = PasswordHash::new(hash)?;

    let is_correct = argon2
        .verify_password(input.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(is_correct)
}
