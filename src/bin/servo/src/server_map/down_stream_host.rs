use std::borrow::Borrow;

use pingora::http::RequestHeader;
use thiserror::Error;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct DownStreamHost(String);

impl Borrow<str> for DownStreamHost {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DownStreamHost {
    fn from(host: &str) -> Self {
        DownStreamHost(host.to_owned())
    }
}

impl From<String> for DownStreamHost {
    fn from(host: String) -> Self {
        DownStreamHost(host)
    }
}

impl TryFrom<&RequestHeader> for DownStreamHost {
    type Error = Error;

    fn try_from(req_header: &RequestHeader) -> Result<Self, Error> {
        let host_header = req_header
            .headers
            .get("host")
            .ok_or(Error::HostDoesNotExistInReqHeader())?;

        let host_header = host_header
            .to_str()
            .map_err(|err| Error::HostHeaderHasInvalidCharecters(err.to_string()))?;

        Ok(DownStreamHost::from(host_header))
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("There is no host header in the req headers")]
    HostDoesNotExistInReqHeader(),

    #[error("The host header has invalid charecters {0}")]
    HostHeaderHasInvalidCharecters(String),
}
