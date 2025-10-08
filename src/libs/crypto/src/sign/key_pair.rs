use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct KeyPair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub creation_time: NaiveDateTime,
}
