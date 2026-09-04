use sqlx::{Connection, PgPool};

use crate::{
    error::{AppError, AppResult},
    models::{CreateRoom, CreateRoomResponse, CurrentPlayerResponse, CurrentRoomResponse},
    repositories::room_repository,
    services::{player_service, session_service},
    utils,
};

const INVITE_CODE_LENGTH: u8 = 6;

pub async fn create_room(pool: &PgPool) -> AppResult<CreateRoomResponse> {
    let invite_code = utils::generate_invite_code(INVITE_CODE_LENGTH);
    let invite_code_hash = utils::sha256_hash(&invite_code);
    let room = CreateRoom { invite_code_hash };

    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let created_room = room_repository::create_room(&mut connection, &room)
        .await
        .map_err(AppError::from)?;

    Ok(CreateRoomResponse {
        id: created_room.id,
        invite_code: invite_code,
    })
}

pub async fn join_room(
    pool: &PgPool,
    session_token: &str,
    attestation_token: &str,
    invite_code: &str,
) -> AppResult<CurrentRoomResponse> {
    // TODO: Validate attestation token

    let invite_code_hash = utils::sha256_hash(invite_code);

    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let room = room_repository::find_active_room(&mut connection, &invite_code_hash)
        .await
        .inspect_err(|err| tracing::error!("{}", err.to_string()))
        .map_err(AppError::from)?;

    let Some(room) = room else {
        return Err(AppError::RoomDoesNotExist);
    };

    if invite_code_hash != room.invite_code_hash {
        return Err(AppError::InvalidInviteCode);
    }

    // Create player
    let mut tx = connection.begin().await.map_err(AppError::from)?;
    let player = player_service::create_or_return_existing_player(&mut tx, room.id, session_token)
        .await
        .map_err(AppError::from)?;
    let session = session_service::create_token_pair(&mut tx, room.id, player.id, session_token)
        .await
        .map_err(AppError::from)?;
    tx.commit().await.map_err(AppError::from)?;

    // TODO: Run job to update player count

    Ok(CurrentRoomResponse {
        id: room.id,
        player_count: room.player_count + 1,
        current_player: CurrentPlayerResponse {
            id: player.id,
            display_name: player.display_name,
            access_token: session.access_token,
            access_token_expires_at: session.access_token_expiry,
            refresh_token: session.refresh_token,
            refresh_token_expires_at: session.refresh_token_expiry,
        },
    })
}
