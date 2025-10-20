use actix_web::{HttpResponse, Responder, get};

#[get("/ping")]
pub async fn get_ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}
