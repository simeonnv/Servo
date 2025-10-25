use std::marker::PhantomData;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::{
    Error,
    jwt::{algoritms::SigAlgoritm, split_jwt::split_jwt},
};

#[cfg(feature = "dilithium3")]
struct Dilithium3;

#[cfg(feature = "falcon512")]
struct Falcon512;

#[derive(Debug)]
pub struct Jwt<AlgType> {
    alg: PhantomData<AlgType>,
    pub head: Box<[u8]>,
    pub body: Box<[u8]>,
    pub sig: Box<[u8]>,
    pub serialized_body: JsonValue,
}

impl<AlgType: SigAlgoritm> Jwt<AlgType> {
    pub fn decode(raw_jwt: &str, public_pem: &[u8]) -> Result<Self, Error> {
        let (head_base64, body_base64, sig_str_base64, sigless_jwt) = split_jwt(raw_jwt)?;

        let sig_str = BASE64_URL_SAFE_NO_PAD
            .decode(sig_str_base64)
            .map_err(|e| Error::InvalidJWT(e.to_string()))?
            .into_boxed_slice();

        let valid_sig = AlgType::verify(sigless_jwt.as_bytes(), &sig_str, public_pem)?;
        if !valid_sig {
            return Err(Error::InvalidJWT("invalid sig".into()));
        }

        let head = BASE64_URL_SAFE_NO_PAD
            .decode(head_base64)
            .map_err(|e| Error::InvalidJWT(e.to_string()))?
            .into_boxed_slice();
        let body = BASE64_URL_SAFE_NO_PAD
            .decode(body_base64)
            .map_err(|e| Error::InvalidJWT(e.to_string()))?
            .into_boxed_slice();

        let serialized_body = serde_json::from_slice(&body)?;

        Ok(Self {
            alg: PhantomData,
            head,
            body,
            serialized_body,
            sig: sig_str,
        })
    }

    pub fn serialize<T: Serialize, S: Serialize>(
        head: T,
        body: S,
        private_pem: &[u8],
    ) -> Result<Self, Error> {
        let head_json = serde_json::to_string(&head)?; // Renamed for clarity
        let body_json = serde_json::to_string(&body)?; // Renamed for clarity
        let serialized_body = serde_json::from_str(&body_json)?;

        let head_base64 = BASE64_URL_SAFE_NO_PAD.encode(&head_json);
        let body_base64 = BASE64_URL_SAFE_NO_PAD.encode(&body_json);

        let sigless_jwt = format!("{head_base64}.{body_base64}");

        let sig = AlgType::encode(sigless_jwt.as_bytes(), private_pem)?;

        let head_bytes = head_json.into_bytes().into_boxed_slice();
        let body_bytes = body_json.into_bytes().into_boxed_slice();

        Ok(Self {
            alg: PhantomData,
            head: head_bytes,
            body: body_bytes,
            serialized_body,
            sig,
        })
    }

    pub fn encode(&self) -> String {
        let head_base64 = BASE64_URL_SAFE_NO_PAD.encode(self.head.clone());
        let body_base64 = BASE64_URL_SAFE_NO_PAD.encode(self.body.clone());
        let sig_base64 = BASE64_URL_SAFE_NO_PAD.encode(self.sig.clone());

        format!("{head_base64}.{body_base64}.{sig_base64}")
    }

    pub fn encode_into(self) -> String {
        let head_base64 = BASE64_URL_SAFE_NO_PAD.encode(self.head);
        let body_base64 = BASE64_URL_SAFE_NO_PAD.encode(self.body);
        let sig_base64 = BASE64_URL_SAFE_NO_PAD.encode(self.sig);

        format!("{head_base64}.{body_base64}.{sig_base64}")
    }
}
