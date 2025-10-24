use crate::Error;

pub trait SigAlgoritm {
    fn verify(input: &[u8], signature: &[u8], key: &[u8]) -> Result<bool, Error>;
    fn encode(input: &[u8], private_pem: &[u8]) -> Result<Box<[u8]>, Error>;
}
