use utoipa::Modify;
use utoipa::OpenApi;

use crate::routes;

use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

struct BearerAuthAddon;
impl Modify for BearerAuthAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::get_ping::get_ping,
        routes::get_public_pem::get_public_pem,
        routes::post_login::post_login,
        routes::post_refresh_session::post_refresh_session,
        routes::post_signup::post_signup,
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
    ),
    modifiers(&BearerAuthAddon),
    security(
        ("bearer_auth" = [])
    )

)]
pub struct ApiDoc;
