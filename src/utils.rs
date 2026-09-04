use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize)]
pub struct Claims {
    // Player ID
    pub sub: String,

    // Room ID
    pub rid: String,

    pub exp: i64,
}

pub fn generate_jwt(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let token = jsonwebtoken::encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Validates whether a given JWT is valid.
/// Note that we should also cross check claims against those provided.
pub fn validate_jwt(
    access_token: String,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let claims = jsonwebtoken::decode::<Claims>(
        access_token.as_bytes(),
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(claims.claims)
}
