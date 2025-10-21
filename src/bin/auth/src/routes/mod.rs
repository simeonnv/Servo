use crate::api_docs;
use actix_web::{dev::HttpServiceFactory, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod get_ping;
pub mod get_public_pem;
pub mod post_login;
pub mod post_refresh_session;
pub mod post_signup;

pub fn routes() -> impl HttpServiceFactory {
    web::scope("")
        .service(get_ping::get_ping)
        .service(get_public_pem::get_public_pem)
        .service(post_login::post_login)
        .service(post_refresh_session::post_refresh_session)
        .service(post_signup::post_signup)
        .service(
            SwaggerUi::new("/swagger/{_:.*}")
                .url("/api-docs/openapi.json", api_docs::ApiDoc::openapi()),
        )
        .service(web::redirect("/swagger", "/swagger/"))
}
