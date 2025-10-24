use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: Uuid,          // User ID
    pub iat: usize,         // when its created (unix timestamp)
    pub exp: usize,         // Expiration time (Unix timestamp)
    pub roles: Vec<String>, // User role
}
