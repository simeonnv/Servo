use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use crypto::sign::falcon512::sign_falcon512::sign_falcon512;
use log::debug;
use serde_json::json;
use uuid::Uuid;

use crate::{Error, jwt::jwt_claims::JWTClaims};

pub async fn create_jwt(
    account_id: Uuid,
    account_role: String,
    jwt_lifetime: Duration,
    private_key: &Vec<u8>,
) -> Result<String, Error> {
    let now = Utc::now().naive_utc();
    let jwt_claims: JWTClaims = JWTClaims {
        sub: account_id,
        exp: (now + jwt_lifetime).and_utc().timestamp() as usize,
        role: account_role,
    };

    let header = json!({
        // "alg": match alg_type {
        //     AlgorithmType::Dilithium3 => "PQ-Dilithium3",
        //     AlgorithmType::Falcon512 => "PQ-FALC512",
        //     AlgorithmType::Rsa => "RS256",
        // },
        "alg": "PQ-FALC512",
        "typ": "JWT"
    })
    .to_string();

    let claims = serde_json::to_string(&jwt_claims)?;

    let base64_header = BASE64_URL_SAFE_NO_PAD.encode(header);
    let base64_claims = BASE64_URL_SAFE_NO_PAD.encode(claims);

    let head_and_body = format!("{}.{}", base64_header, base64_claims);
    let head_and_body_bytes = format!("{}.{}", base64_header, base64_claims).into_bytes();

    let signature = sign_falcon512(&head_and_body_bytes, private_key)?;

    let base64_signature = BASE64_URL_SAFE_NO_PAD.encode(signature);

    let jwt = format!("{}.{}", head_and_body, base64_signature);

    debug!("generated jwt: {}", &jwt);

    Ok(jwt)
}
