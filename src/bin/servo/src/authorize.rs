use std::collections::HashSet;

use crate::server_map::UpstreamAuth;
use chrono::Utc;
use log::debug;
use pingora::http::RequestHeader;
use servo_auth::jwt::{Jwt, algoritms::Rsa};
use thiserror::Error;

pub fn authorize(
    req_header: &RequestHeader,
    upstream_auth: &UpstreamAuth,
) -> Result<(), AuthError> {
    if !upstream_auth.jwt_required {
        return Ok(());
    }

    let auth_header = req_header
        .headers
        .get("Authorization")
        .ok_or_else(|| AuthError::InvalidAuthHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidAuthHeader)?;

    let jwt = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AuthError::InvalidAuthHeader)?
        .trim();
    dbg!(jwt);

    let public_pem = upstream_auth.public_pem_sync.get_public_pem();

    let jwt = Jwt::<Rsa>::decode(jwt, public_pem.as_bytes()).map_err(|err| {
        debug!("jwt decode error: {err}");
        AuthError::InvalidJWT
    })?;

    let exp = jwt
        .serialized_body
        .get("exp")
        .ok_or_else(|| AuthError::InvalidJWT)?
        .as_u64()
        .ok_or_else(|| AuthError::InvalidJWT)?;

    let now = Utc::now().naive_utc().and_utc().timestamp() as u64;

    if now > exp {
        return Err(AuthError::JWTExpired);
    }

    let allowed_roles = match &upstream_auth.jwt_auth_roles {
        Some(e) => e,
        None => return Ok(()),
    };

    let roles = jwt
        .serialized_body
        .get("roles")
        .ok_or_else(|| AuthError::InvalidJWT)?
        .as_array()
        .ok_or_else(|| AuthError::InvalidJWT)?;

    let mut token_roles = HashSet::with_capacity(roles.len() * 2);
    for role in roles {
        let role = role.as_str();
        let role = match role {
            Some(e) => e,
            None => continue,
        };
        token_roles.insert(role);
    }

    let role_authorized = {
        let mut authorized: bool = false;
        for token_role in token_roles {
            if allowed_roles.contains(token_role) {
                authorized = true;
            }
        }
        authorized
    };

    if !role_authorized {
        return Err(AuthError::InvalidJWT);
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("No Auth Header")]
    InvalidAuthHeader,

    #[error("invalid jwt")]
    InvalidJWT,

    #[error("jwt expired")]
    JWTExpired,
}
