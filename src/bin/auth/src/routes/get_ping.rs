use actix_web::{HttpResponse, Responder, get};

#[utoipa::path(
    get,
    path = "/ping",
    responses(
        (status = 200, body = String, example = "pong"),
    ),
    tag = "Health"
)]
#[get("/ping")]
pub async fn get_ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}
