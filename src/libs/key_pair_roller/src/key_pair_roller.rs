use log::{error, info};
use servo_crypto::Error as CryptoError;
use servo_crypto::sign::key_pair::KeyPair;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{sync::watch::*, task::JoinHandle, time::sleep};

pub struct Key {
    rx: Receiver<Arc<[u8]>>,
    tx: Sender<Arc<[u8]>>,
}

impl Key {
    pub fn new(key: Arc<[u8]>) -> Self {
        let (tx, rx) = channel(key);
        Self { tx, rx }
    }

    pub fn get_key(&self) -> Arc<[u8]> {
        self.rx.borrow().clone()
    }

    pub fn get_sender(&self) -> Sender<Arc<[u8]>> {
        self.tx.clone()
    }
}

pub struct KeyPairRoller {
    public_key: Key,
    private_key: Key,
    pub roll_inteval: Duration,
    task: JoinHandle<()>,
}

impl Drop for KeyPairRoller {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl KeyPairRoller {
    pub fn get_public_key(&self) -> Arc<[u8]> {
        self.public_key.get_key()
    }
    pub fn get_private_key(&self) -> Arc<[u8]> {
        self.private_key.get_key()
    }
}

impl KeyPairRoller {
    pub fn init_rsa_roller(roll_inteval: Duration) -> Result<Self, Error> {
        use servo_crypto::sign::rsa::generate_rsa_key_pair::generate_rsa_key_pair;
        let key_pair = generate_rsa_key_pair()?;

        let public_key: Arc<[u8]> = Arc::from(key_pair.public_key);
        let private_key: Arc<[u8]> = Arc::from(key_pair.private_key);

        let public_key = Key::new(public_key);
        let private_key = Key::new(private_key);

        let public_key_tx = public_key.get_sender();
        let private_key_tx = private_key.get_sender();

        let task = tokio::spawn(background_key_pair_roller(
            generate_rsa_key_pair,
            roll_inteval.clone(),
            public_key_tx,
            private_key_tx,
        ));

        Ok(Self {
            public_key,
            private_key,
            roll_inteval,
            task,
        })
    }
}

async fn background_key_pair_roller<T: Fn() -> Result<KeyPair, CryptoError>>(
    key_pair_generator: T,
    roll_inteval: Duration,
    pub_key_tx: Sender<Arc<[u8]>>,
    priv_key_tx: Sender<Arc<[u8]>>,
) {
    loop {
        sleep(roll_inteval).await;
        info!("generating new keypair");
        let key_pair = key_pair_generator();
        let key_pair = match key_pair {
            Ok(e) => e,
            Err(err) => {
                error!(
                    "failed to init key pair in backround roller thread: {err}, will retry later"
                );
                continue;
            }
        };
        let pub_key: Arc<[u8]> = Arc::from(key_pair.public_key);
        let priv_key: Arc<[u8]> = Arc::from(key_pair.private_key);

        if let Err(err) = pub_key_tx.send(pub_key) {
            error!("failed to send public key in roller smh: {err}");
        };
        if let Err(err) = priv_key_tx.send(priv_key) {
            error!("failed to send private key in roller smh: {err}");
        };
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Crypto error at key pair roller => {0}")]
    CryptoError(#[from] servo_crypto::Error),
}
