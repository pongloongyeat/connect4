use rand::{RngExt, distr::Alphanumeric};
use sha2::{Digest, Sha256};

/// Gets env var, or None if not available.
pub fn get_env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

pub fn generate_invite_code(length: u8) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect()
}

pub fn sha256_hash(s: &str) -> String {
    let hash = Sha256::digest(s.as_bytes());
    hex::encode(hash)
}
