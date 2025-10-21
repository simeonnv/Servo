use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{Error, tables::Accounts};

pub async fn get_account_by_id(
    account_id: Uuid,
    db_pool: &Pool<Postgres>,
) -> Result<Accounts, Error> {
    let db_res: Option<Accounts> = sqlx::query_as!(
        Accounts,
        r#"
            SELECT * FROM Accounts
                WHERE account_id = $1
            ;
        "#,
        account_id
    )
    .fetch_optional(db_pool)
    .await?;

    match db_res {
        Some(account) => Ok(account),
        None => Err(Error::InvalidAccount(
            "There is no user with such id!".into(),
        )),
    }
}
