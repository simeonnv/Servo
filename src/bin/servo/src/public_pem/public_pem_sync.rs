use log::warn;
use std::{path::PathBuf, time::Duration};
use tokio::{
    sync::watch::{self, Receiver, Sender},
    task::JoinHandle,
    time::sleep,
};
use url::Url;

use crate::{
    config_toml::{AuthToml, PublicPemLocationToml},
    public_pem::{Error, PublicPem},
};

#[derive(Debug)]
pub struct PublicPemSync {
    #[allow(unused)]
    public_pem_sender: Sender<PublicPem>,
    public_pem_reciever: Receiver<PublicPem>,
    task_handle: Option<JoinHandle<()>>,
    pub update_duration: Duration,
}

impl PublicPemSync {
    pub async fn init_auth_toml(auth_toml: &AuthToml) -> Result<Self, Error> {
        let update_duration = Duration::from_millis(auth_toml.check_duration);
        match &auth_toml.public_pem_location {
            PublicPemLocationToml::PublicPemHttpUrl(url) => {
                Self::init_from_http_url(url, update_duration).await
            }
            PublicPemLocationToml::PublicPemPath(path) => Self::init_from_path(path),
        }
    }
}

impl Drop for PublicPemSync {
    fn drop(&mut self) {
        if let Some(handle) = self.task_handle.as_ref() {
            handle.abort();
        }
    }
}

impl PublicPemSync {
    pub fn get_public_pem(&self) -> PublicPem {
        self.public_pem_reciever.borrow().clone()
    }
}

impl PublicPemSync {
    pub async fn init_from_http_url(url: &Url, update_duration: Duration) -> Result<Self, Error> {
        let public_pem = PublicPem::from_http_req(&url).await?;
        let (public_pem_sender, public_pem_reciever) = watch::channel(public_pem);

        let task_handle = tokio::spawn(background_public_pem_sync_url(
            url.clone(),
            public_pem_sender.clone(),
            update_duration,
        ));

        Ok(Self {
            public_pem_sender,
            public_pem_reciever,
            task_handle: Some(task_handle),
            update_duration,
        })
    }

    pub fn init_from_path(path: &PathBuf) -> Result<Self, Error> {
        let public_pem = PublicPem::from_path(path)?;
        let (public_pem_sender, public_pem_reciever) = watch::channel(public_pem);

        Ok(Self {
            public_pem_sender,
            public_pem_reciever,
            task_handle: None,
            update_duration: Duration::default(),
        })
    }
}

async fn background_public_pem_sync_url(
    url: Url,
    public_pem_sender: Sender<PublicPem>,
    update_duration: Duration,
) {
    loop {
        sleep(update_duration).await;
        let public_pem = PublicPem::from_http_req(&url).await;
        let public_pem = match public_pem {
            Ok(e) => e,
            Err(err) => {
                warn!("failed to fetch public pem: {err} will retry later...");
                continue;
            }
        };
        let _ = public_pem_sender.send(public_pem);
    }
}
