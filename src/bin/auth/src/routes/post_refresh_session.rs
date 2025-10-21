use actix_web::{
    HttpResponse, post,
    web::{self, Data},
};

use key_pair_roller::KeyPairRoller;
use serde::{Deserialize, Serialize};
use servo_auth::{
    jwt::create_jwt::create_jwt,
    refresh_token::get_refresh_token_data_db::get_refresh_token_data_db,
};
use sqlx::{Pool, Postgres};
use utoipa::ToSchema;

use crate::{Error, config::JWT_LIFETIME};

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(as = Post::Auth::RefreshSession::Req)]
pub struct Req {
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(as = Post::RefreshSession::Res)]
struct Res {
    status: &'static str,
    data: DataRes,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(as = Post::RefreshSession::Res::DataRes)]
struct DataRes {
    jwt: String,
}

#[utoipa::path(
    post,
    path = "/refresh_session",
    request_body = Req,
    responses(),
    security(),
    tag = "Auth"
)]
#[post("/refresh_session")]
pub async fn post_refresh_session(
    body: web::Json<Req>,
    db_pool: web::Data<Pool<Postgres>>,
    key_pair_roller: Data<KeyPairRoller>,
) -> Result<HttpResponse, Error> {
    let token_data = get_refresh_token_data_db(&body.refresh_token, &db_pool).await?;
    let private_key = key_pair_roller.get_private_key();
    let jwt = create_jwt(
        token_data.account_id,
        "user".into(),
        JWT_LIFETIME,
        &private_key,
    )
    .await?;

    return Ok(HttpResponse::Ok().json(Res {
        status: "success",
        data: DataRes { jwt },
    }));
}
