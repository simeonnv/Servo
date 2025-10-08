use sqlx::{Pool, Postgres};

use crate::Error;

pub async fn delete_refresh_token_db(
    refresh_token: &String,
    db_pool: &Pool<Postgres>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
            DELETE FROM RefreshTokens WHERE refresh_token = $1;
        "#,
    )
    .bind(refresh_token)
    .execute(db_pool)
    .await?;

    Ok(())
}
