use argon2::Params;

mod rand_string;
pub use rand_string::rand_string;

mod error;
pub use error::Error;

#[cfg(feature = "argon2")]
pub mod hashing;

pub mod kem;
pub mod sign;

pub const ARGON2_PARAMS: Params = {
    let params = Params::new(
        8192, // Memory cost
        1,    // Iterations
        2,    // Parallelism
        None, // Optional output length (None uses default)
    );
    match params {
        Ok(e) => e,
        Err(_) => {
            panic!("INVALID ARGON2 PARAMS!!!");
        }
    }
};
