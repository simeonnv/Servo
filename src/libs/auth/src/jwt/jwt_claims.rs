use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: Uuid,    // User ID
    pub exp: usize,   // Expiration time (Unix timestamp)
    pub role: String, // User role
}
