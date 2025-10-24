use crate::Error;

pub fn split_jwt(raw_jwt: &str) -> Result<(&str, &str, &str, &str), Error> {
    let mut jwt_parts = raw_jwt.splitn(3, '.');

    let head = jwt_parts
        .next()
        .ok_or(Error::InvalidJWT("jwt has no head".into()))?;

    let body = jwt_parts
        .next()
        .ok_or(Error::InvalidJWT("jwt has no head".into()))?;

    let sig_str = jwt_parts
        .next()
        .ok_or(Error::InvalidJWT("jwt has no sig".into()))?;

    let (prefix, _) = raw_jwt.rsplit_once('.').unwrap();

    Ok((head, body, sig_str, prefix))
}
