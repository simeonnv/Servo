use rand::{Rng, distr::Alphanumeric};

pub fn rand_string(len: usize) -> String {
    let random_string: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();

    random_string
}
