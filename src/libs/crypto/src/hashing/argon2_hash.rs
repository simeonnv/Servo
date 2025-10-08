use argon2::{
    Algorithm, Argon2, Version,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::{ARGON2_PARAMS, Error};

pub async fn argon2_hash(input: &String) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, ARGON2_PARAMS);
    let password_hash = argon2.hash_password(input.as_bytes(), &salt)?.to_string();

    Ok(password_hash)
}
