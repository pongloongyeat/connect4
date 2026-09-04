use chrono::{DateTime, Utc};

use crate::utils::Claims;

pub mod connect4_engine;
pub mod game_service;
pub mod player_service;
pub mod room_service;
pub mod session_service;

pub const DEFAULT_LIMIT: i32 = 20;

pub struct SessionDetails {
    pub claims: Claims,
    pub access_token: String,
    pub access_token_expiry: DateTime<Utc>,
    pub refresh_token: String,
    pub refresh_token_expiry: DateTime<Utc>,
}
