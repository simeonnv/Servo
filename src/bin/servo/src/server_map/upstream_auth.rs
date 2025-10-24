use std::sync::Arc;

use crate::public_pem::PublicPemSync;

#[derive(Debug)]
pub struct UpstreamAuth {
    pub public_pem_sync: Arc<PublicPemSync>,
    pub jwt_required: bool,
    pub jwt_auth_roles: Option<Vec<String>>,
}
