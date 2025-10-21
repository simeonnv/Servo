use sqlx::{Pool, Postgres};

use crate::Error;

pub async fn does_account_exist_db(
    username: &String,
    db_pool: &Pool<Postgres>,
) -> Result<bool, Error> {
    let account_count = sqlx::query_scalar!(
        r#"
            SELECT COUNT(*) AS count
                FROM Accounts
                WHERE username = $1
            ;
        "#,
        username
    )
    .bind(username)
    .fetch_one(db_pool)
    .await?
    .unwrap_or(0);

    Ok(account_count > 0)
}
