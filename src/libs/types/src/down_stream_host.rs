use std::borrow::{Borrow, Cow};

use pingora_http::RequestHeader;
use thiserror::Error;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct DownStreamHost<'a>(Cow<'a, str>);

impl<'a> DownStreamHost<'a> {
    pub fn into_owned_host(self) -> DownStreamHost<'static> {
        let owned_cow: Cow<'static, str> = Cow::Owned(self.0.into_owned());
        DownStreamHost(owned_cow)
    }

    pub fn into_owned_string(self) -> String {
        self.0.into_owned()
    }
}

impl<'a> Borrow<str> for DownStreamHost<'a> {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl<'a> From<&'a str> for DownStreamHost<'a> {
    fn from(host: &'a str) -> Self {
        DownStreamHost(Cow::Borrowed(host))
    }
}

impl From<String> for DownStreamHost<'static> {
    fn from(host: String) -> Self {
        DownStreamHost(Cow::Owned(host))
    }
}

impl<'a, 'b> TryFrom<&'b RequestHeader> for DownStreamHost<'b> {
    type Error = Error;

    fn try_from(req_header: &'b RequestHeader) -> Result<Self, Error> {
        let host_header = req_header
            .headers
            .get("host")
            .ok_or(Error::HostDoesNotExistInReqHeader())?;

        let host_header = host_header
            .to_str()
            .map_err(|err| Error::HostHeaderHasInvalidCharecters(err.to_string()))?;

        Ok(DownStreamHost(Cow::Borrowed(host_header)))
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("There is no host header in the req headers")]
    HostDoesNotExistInReqHeader(),

    #[error("The host header has invalid charecters {0}")]
    HostHeaderHasInvalidCharecters(String),
}
