use chrono::{Duration, Utc};
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

use crate::{
    models::{AppError, AppResult},
    repositories::refresh_token_repository,
    services::SessionDetails,
    utils::{self, Claims},
};

/// Revokes all previous refresh tokens and returns a new
/// token pair.
pub async fn create_token_pair(
    connection: &mut PgConnection,
    room_id: i64,
    player_id: i64,
    session_token: &str,
) -> AppResult<SessionDetails> {
    let jwt_secret = utils::get_env("JWT_SECRET").ok_or_else(|| {
        tracing::error!("Missing JWT_SECRET env");
        AppError::InternalError(None)
    })?;

    let access_token_expiry = utils::get_env("ACCESS_TOKEN_EXPIRY").ok_or_else(|| {
        tracing::error!("Missing ACCESS_TOKEN_EXPIRY env");
        AppError::InternalError(None)
    })?;
    let access_token_expiry_seconds = i64::from_str_radix(&access_token_expiry, 10)
        .inspect_err(|err| tracing::error!("Invalid ACCESS_TOKEN_EXPIRY value: {err}"))
        .map_err(|_| AppError::InternalError(None))?;

    let refresh_token_expiry =
        utils::get_env("REFRESH_TOKEN_EXPIRY").ok_or_else(|| AppError::InternalError(None))?;
    let refresh_token_expiry_seconds = i64::from_str_radix(&refresh_token_expiry, 10)
        .inspect_err(|err| tracing::error!("Invalid REFRESH_TOKEN_EXPIRY value: {err}"))
        .map_err(|_| AppError::InternalError(None))?;

    let access_token_expiry = Utc::now() + Duration::seconds(access_token_expiry_seconds);
    let claims = Claims {
        sub: player_id.to_string(),
        rid: room_id.to_string(),
        exp: access_token_expiry.timestamp(),
    };

    let access_token = utils::generate_jwt(&claims, &jwt_secret)
        .inspect_err(|err| tracing::error!("Failed to generate JWT token: {}", err))
        .map_err(|_| AppError::InternalError(None))?;

    let session_hash = utils::sha256_hash(&session_token);
    let refresh_token = Uuid::new_v4().to_string();
    let refresh_token_hash = utils::sha256_hash(&refresh_token);
    let refresh_token_expiry = Utc::now() + Duration::seconds(refresh_token_expiry_seconds);
    let mut tx = connection.begin().await.map_err(AppError::from)?;
    let _ = refresh_token_repository::revoke_all_by_session_hash(&mut tx, &session_hash)
        .await
        .map_err(AppError::from)?;
    let _ = refresh_token_repository::create_refresh_token(
        &mut tx,
        &refresh_token_hash,
        room_id,
        player_id,
        refresh_token_expiry,
    )
    .await
    .map_err(AppError::from)?;
    tx.commit().await.map_err(AppError::from)?;

    Ok(SessionDetails {
        claims,
        access_token,
        access_token_expiry,
        refresh_token,
        refresh_token_expiry,
    })
}
