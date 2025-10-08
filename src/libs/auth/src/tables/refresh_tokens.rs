use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
pub struct RefreshTokens {
    pub refresh_token_id: Uuid,
    pub account_id: Uuid,
    pub refresh_token: String,
    pub role: String,
    pub created_at: NaiveDateTime,
}
