use sqlx::{Connection, PgConnection};

use crate::{
    error::{AppError, AppResult},
    models::CurrentPlayer,
    repositories::{player_repository, refresh_token_repository},
    utils,
};

/// Creates or returns an existing player.
pub async fn create_or_return_existing_player(
    connection: &mut PgConnection,
    room_id: i64,
    session_token: &str,
) -> AppResult<CurrentPlayer> {
    let session_hash = utils::sha256_hash(&session_token);
    let player_id = refresh_token_repository::find_player_id(connection, &session_hash, room_id)
        .await
        .map_err(AppError::from)?;

    if let Some(player_id) = player_id {
        let player = player_repository::find_player_by_id(connection, player_id)
            .await
            .map_err(AppError::from)?;

        if let Some(player) = player {
            return Ok(player);
        } else {
            tracing::warn!("Player {player_id} not found but refresh token exists");
        }
    }

    let mut tx = connection.begin().await.map_err(AppError::from)?;
    let ref_no = player_repository::player_ref_no(&mut tx)
        .await
        .map_err(AppError::from)?;
    let player = player_repository::create_player(&mut tx, room_id, ref_no)
        .await
        .inspect_err(|err| tracing::error!("{}", err.to_string()))
        .map_err(AppError::from)?;
    tx.commit()
        .await
        .inspect_err(|err| tracing::error!("{}", err.to_string()))
        .map_err(AppError::from)?;

    Ok(player)
}
