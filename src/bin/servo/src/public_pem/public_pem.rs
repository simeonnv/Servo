use std::{borrow::Borrow, fs, path::PathBuf, sync::Arc};
use url::Url;

use crate::public_pem::Error;

#[derive(Debug)]
pub struct PublicPem(Arc<[u8]>);

impl PublicPem {
    pub async fn from_http_req(url: &Url) -> Result<Self, Error> {
        let res = reqwest::get(url.as_str()).await?;
        let pub_pem = res.bytes().await?.to_vec();
        let pub_pem: Arc<[u8]> = Arc::from(pub_pem.into_boxed_slice());

        Ok(Self(pub_pem))
    }

    pub fn from_path(path: &PathBuf) -> Result<Self, Error> {
        let file =
            fs::read(path).map_err(|err| Error::FailedToReadPublicPemFromFS(err.to_string()))?;
        let pub_pem: Arc<[u8]> = Arc::from(file.into_boxed_slice());

        Ok(Self(pub_pem))
    }
}

impl Borrow<Arc<[u8]>> for PublicPem {
    fn borrow(&self) -> &Arc<[u8]> {
        &self.0
    }
}

impl Borrow<[u8]> for PublicPem {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl Clone for PublicPem {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
