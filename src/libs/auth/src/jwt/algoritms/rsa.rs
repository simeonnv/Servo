use crate::{Error, jwt::algoritms::SigAlgoritm};

#[derive(Debug)]
pub struct Rsa;

impl SigAlgoritm for Rsa {
    fn verify(input: &[u8], signature: &[u8], public_pem: &[u8]) -> Result<bool, Error> {
        use servo_crypto::sign::rsa::validate_rsa_sign::validate_rsa_sign;
        Ok(validate_rsa_sign(input, signature, public_pem)?)
    }

    fn encode<'a>(input: &'a [u8], private_pem: &[u8]) -> Result<Box<[u8]>, Error> {
        use servo_crypto::sign::rsa::sign_rsa::sign_rsa;
        Ok(sign_rsa(input, private_pem)?)
    }
}
