use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
pub struct KeyPairs {
    pub key_pair_id: Uuid,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub created_at: NaiveDateTime,
}
