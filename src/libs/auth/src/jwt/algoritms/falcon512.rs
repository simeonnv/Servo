use crate::{Error, jwt::algoritms::SigAlgoritm};

#[derive(Debug)]
pub struct Falcon512;

impl SigAlgoritm for Falcon512 {
    fn verify(input: &[u8], signature: &[u8], public_pem: &[u8]) -> Result<bool, Error> {
        use servo_crypto::sign::falcon512::validate_falcon512_sign::validate_falcon512_sign;
        Ok(validate_falcon512_sign(input, signature, public_pem)?)
    }

    fn encode<'a>(input: &'a [u8], private_pem: &[u8]) -> Result<Box<[u8]>, Error> {
        use servo_crypto::sign::falcon512::sign_falcon512::sign_falcon512;
        Ok(sign_falcon512(input, private_pem)?)
    }
}
