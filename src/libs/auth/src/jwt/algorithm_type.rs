#[derive(Debug, Clone, Copy)]
pub enum AlgorithmType {
    Rsa,
    Dilithium3,
    Falcon512,
}

impl AlgorithmType {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Rsa => "rsa",
            Self::Dilithium3 => "dilithium3",
            Self::Falcon512 => "falcon512",
        }
    }
}
