use crate::{Error, jwt::algoritms::SigAlgoritm};

#[derive(Debug)]
pub struct Dilithium3;

impl SigAlgoritm for Dilithium3 {
    fn verify(input: &[u8], signature: &[u8], public_pem: &[u8]) -> Result<bool, Error> {
        use servo_crypto::sign::dilithium3::validate_dilithium3_sign::validate_dilithium3_sign;
        Ok(validate_dilithium3_sign(input, signature, public_pem)?)
    }

    fn encode<'a>(input: &'a [u8], private_pem: &[u8]) -> Result<Box<[u8]>, Error> {
        use servo_crypto::sign::dilithium3::sign_dilithium3::sign_dilithium3;
        Ok(sign_dilithium3(input, private_pem)?)
    }
}
