use actix_web::{
    HttpResponse, post,
    web::{self, Data},
};

use key_pair_roller::KeyPairRoller;
use serde::{Deserialize, Serialize};
use servo_account::query::{create_account_db, does_account_exist_db};
use servo_auth::refresh_token::create_refresh_token_db::create_refresh_token_db;
use sqlx::{Pool, Postgres};
use utoipa::ToSchema;

use crate::{
    Error,
    config::{MAX_PASSWORD_LENGHT, MAX_USERNAME_LENGHT, MIN_PASSWORD_LENGHT, MIN_USERNAME_LENGHT},
    generate_jwt,
};

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(as = Post::Signup::Req)]
pub struct Req {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(as = Post::Signup::Res)]
struct Res {
    status: &'static str,
    data: DataRes,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(as = Post::Signup::Res::DataRes)]
struct DataRes {
    refresh_token: String,
    jwt: String,
}

#[utoipa::path(
    post,
    path = "/signup",
    request_body = Req,
    security(),
    tag = "Auth"
)]
#[post("/signup")]
pub async fn post_signup(
    body: web::Json<Req>,
    db_pool: Data<Pool<Postgres>>,
    key_pair_roller: Data<KeyPairRoller>,
) -> Result<HttpResponse, Error> {
    if !(MIN_USERNAME_LENGHT..=MAX_USERNAME_LENGHT).contains(&body.username.len()) {
        return Err(Error::BadRequest(format!(
            "Username should be between {MIN_USERNAME_LENGHT} and {MAX_USERNAME_LENGHT}"
        )));
    }
    if !(MIN_PASSWORD_LENGHT..=MAX_PASSWORD_LENGHT).contains(&body.password.len()) {
        return Err(Error::BadRequest(format!(
            "Password should be between {MIN_PASSWORD_LENGHT} and {MAX_PASSWORD_LENGHT}"
        )));
    }

    if does_account_exist_db(&body.username, &db_pool).await? {
        return Err(Error::Conflict("Account already exists".to_string()));
    }

    let account_id = create_account_db(&body.username, &body.password, "user", &db_pool).await?;

    let refresh_token = create_refresh_token_db(&account_id, &"user".to_string(), &db_pool).await?;
    let private_pem = key_pair_roller.get_private_key();
    let jwt = generate_jwt(account_id, vec!["user".into()], &private_pem)?;

    return Ok(HttpResponse::Ok().json(Res {
        status: "success",
        data: DataRes { refresh_token, jwt },
    }));
}
