use chrono::Utc;
use serde_json::json;
use servo_auth::jwt::{Jwt, algoritms::Rsa};
use uuid::Uuid;

use crate::{Error, JWTClaims, config::JWT_LIFETIME};

pub fn generate_jwt(
    account_id: Uuid,
    roles: Vec<String>,
    private_pem: &[u8],
) -> Result<String, Error> {
    let now = Utc::now().naive_utc();
    let claims = JWTClaims {
        exp: (now + JWT_LIFETIME).and_utc().timestamp() as usize,
        iat: now.and_utc().timestamp() as usize,
        sub: account_id,
        roles,
    };
    let header = json!({
        "alg": "RS256",
        "typ": "JWT"
    });

    let jwt = Jwt::<Rsa>::serialize(header, claims, &private_pem)?;
    Ok(jwt.encode_into())
}
