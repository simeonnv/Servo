use crypto::sign::key_pair::KeyPair;
use log::info;
use sqlx::{Pool, Postgres, types::Uuid};

use crate::Error;

pub async fn save_key_pair_db(key_pair: &KeyPair, pool: &Pool<Postgres>) -> Result<(), Error> {
    sqlx::query!(
        r#"
            INSERT INTO KeyPairs
                (key_pair_id, private_key, public_key)
            VALUES ($1, $2, $3);
        "#,
        Uuid::new_v4(),
        key_pair.private_key.clone(),
        key_pair.public_key.clone()
    )
    .execute(pool)
    .await?;

    info!("succesfully saved rsa key pair into db!");

    Ok(())
}
