use actix_web::{HttpResponse, Responder, get, web::Data};
use key_pair_roller::KeyPairRoller;

#[get("/public_pem")]
pub async fn get_public_pem(key_pair_roller: Data<KeyPairRoller>) -> impl Responder {
    let public_key = key_pair_roller.get_public_key();
    HttpResponse::Ok().body(public_key.to_vec())
}
