use chrono::NaiveDateTime;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::Error;

#[derive(sqlx::FromRow, Debug)]
pub struct TokenData {
    pub account_id: Uuid,
    pub username: String,
    pub role: String,
    pub refresh_token: String,
    pub refresh_token_creation_date: NaiveDateTime,
    pub account_creation_date: NaiveDateTime,
}

pub async fn get_refresh_token_data_db(
    refresh_token: &String,
    db_pool: &Pool<Postgres>,
) -> Result<TokenData, Error> {
    let db_res: Option<TokenData> = sqlx::query_as!(
        TokenData,
        r#"
            SELECT
                RefreshTokens.refresh_token,
                RefreshTokens.role,
                RefreshTokens.created_at AS refresh_token_creation_date,
                Accounts.username,
                Accounts.account_id AS account_id,
                Accounts.created_at AS account_creation_date
            FROM
                RefreshTokens
            INNER JOIN Accounts ON
                RefreshTokens.account_id = Accounts.account_id
            WHERE refresh_token = $1;
        "#,
        refresh_token,
    )
    .fetch_optional(db_pool)
    .await?;

    let token_info = match db_res {
        Some(e) => e,
        None => {
            return Err(Error::InvalidRefreshToken("Invalid refresh token!".into()));
        }
    };

    Ok(token_info)
}
