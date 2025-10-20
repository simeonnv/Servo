use log::{info, warn};
use servo_crypto::sign::key_pair::KeyPair;
use sqlx::{Pool, Postgres};

use crate::Error;
use crate::tables::KeyPairs;

pub async fn get_latest_key_pair_db(pool: &Pool<Postgres>) -> Result<Option<KeyPair>, Error> {
    let latest_key_pair: Option<KeyPairs> = sqlx::query_as!(
        KeyPairs,
        r#"
            SELECT * FROM KeyPairs ORDER BY created_at DESC LIMIT 1;
        "#,
    )
    .fetch_optional(pool)
    .await?;

    let latest_key_pair = latest_key_pair.map(|e| KeyPair {
        private_key: e.private_key.into_boxed_slice(),
        public_key: e.public_key.into_boxed_slice(),
        creation_time: e.created_at,
    });

    match latest_key_pair {
        Some(_) => info!("succesfully got latest rsa key pair from db!"),
        None => warn!("Searched, but found no rsa key pair in the db"),
    }
    Ok(latest_key_pair)
}
