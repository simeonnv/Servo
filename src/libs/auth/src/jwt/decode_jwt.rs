use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use crypto::sign::falcon512::validate_falcon512_sign::validate_falcon512_sign;

use crate::{Error, jwt::jwt_claims::JWTClaims};

pub async fn decode_jwt(jwt: &String, public_key: &Vec<u8>) -> Result<JWTClaims, Error> {
    let jwt_parts: Vec<&str> = jwt.splitn(3, '.').collect();

    if jwt_parts.len() != 3 {
        return Err(Error::InvalidJWT(
            "invalid jwt: the jwt doesnt contain 3 fragmets/dots".into(),
        ));
    }

    let head_base64 = jwt_parts[0];
    let body_base64 = jwt_parts[1];
    let sign = BASE64_URL_SAFE_NO_PAD
        .decode(jwt_parts[2])
        .map_err(|e| Error::InvalidJWT(e.to_string()))?;

    let head_and_body = format!("{}.{}", head_base64, body_base64);

    validate_falcon512_sign(&head_and_body.into_bytes(), &sign, public_key)
        .map_err(|e| Error::InvalidJWT(e.to_string()))?;

    let body = BASE64_URL_SAFE_NO_PAD.decode(body_base64)?;

    let claims: JWTClaims = serde_json::from_slice(&body)?;

    Ok(claims)
}
