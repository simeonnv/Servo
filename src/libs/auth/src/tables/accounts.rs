use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
pub struct Accounts {
    pub account_id: Uuid,
    pub username: String,
    pub password: String,
    pub role: String,
    pub created_at: NaiveDateTime,
}
