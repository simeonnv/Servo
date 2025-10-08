use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigToml {
    pub name: String,
    pub listens: Vec<String>,
    pub upstream: Vec<Upstream>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Upstream {
    pub address: String,
}

impl Default for ConfigToml {
    fn default() -> Self {
        let upstream = Upstream {
            address: "somecontianer:33333".into(),
        };

        Self {
            name: "give me a name vro".into(),
            listens: vec!["0.0.0.0:54321".into()],
            upstream: vec![upstream],
        }
    }
}
