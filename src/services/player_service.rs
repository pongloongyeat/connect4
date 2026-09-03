use redis::{AsyncTypedCommands, aio::ConnectionManager};
use sqlx::{Connection, PgPool};

use crate::{
    models::{
        AppError, AppResult, CurrentPlayerResponse, CurrentRoomResponse, JoinRoomRequest, MakeMove,
    },
    repositories::{player_repository, room_repository},
    utils,
};

async fn get_current_player_id(
    redis: &mut ConnectionManager,
    pool: PgPool,
    room_id: i64,
    session_token: String,
) -> AppResult<i64> {
    let player_id = redis
        .get_int(format!("room:{room_id}:sessions:{session_token}"))
        .await
        .inspect_err(|err| tracing::error!("{}", err.to_string()))
        .map_err(AppError::from)?;
    if let Some(player_id) = player_id {
        return Ok(player_id as i64);
    }

    // Fallback to DB
    let session_hash = utils::sha256_hash(&session_token);
    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let player = player_repository::find_player(&mut connection, room_id, session_hash)
        .await
        .inspect_err(|err| tracing::error!("{}", err.to_string()))
        .map_err(AppError::from)?;
    if let Some(player) = player {
        Ok(player.id)
    } else {
        Err(AppError::PlayerDoesNotExist)
    }
}

pub async fn join_room(
    pool: PgPool,
    room_id: i64,
    session_token: String,
    request: JoinRoomRequest,
) -> AppResult<CurrentRoomResponse> {
    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let room = room_repository::find_active_room(&mut connection, room_id)
        .await
        .inspect_err(|err| tracing::error!("{}", err.to_string()))
        .map_err(AppError::from)?;
    let Some(room) = room else {
        return Err(AppError::RoomDoesNotExist);
    };

    if room.is_private {
        let token_hash = room.token_hash;
        let Some(token_hash) = token_hash else {
            tracing::error!("Missing token hash for private room {}", room_id);
            return Err(AppError::InternalError(Some(
                "room has missing token hash.",
            )));
        };

        let Some(invite_code) = request.invite_code else {
            return Err(AppError::MissingInviteCode);
        };
        let invite_code_hash = utils::sha256_hash(&invite_code);
        if invite_code_hash != token_hash {
            return Err(AppError::InvalidInviteCode);
        }
    }

    // Create player
    let session_hash = utils::sha256_hash(&session_token);
    let mut tx = connection.begin().await.map_err(AppError::from)?;
    // TODO: Session token should be hashed?
    let player = player_repository::create_player(&mut tx, room_id, session_hash)
        .await
        .inspect_err(|err| tracing::error!("{}", err.to_string()))
        .map_err(AppError::from)?;
    tx.commit()
        .await
        .inspect_err(|err| tracing::error!("{}", err.to_string()))
        .map_err(AppError::from)?;

    // TODO: Run job to update player count

    Ok(CurrentRoomResponse {
        id: room_id,
        display_name: room.display_name,
        player_count: room.player_count + 1,
        current_player: CurrentPlayerResponse {
            id: player.id,
            display_name: player.display_name,
        },
    })
}

pub async fn make_move(
    redis: &mut ConnectionManager,
    pool: PgPool,
    room_id: i64,
    session_token: String,
    player_move: MakeMove,
) -> AppResult<()> {
    let player_id = get_current_player_id(redis, pool, room_id, session_token).await?;

    todo!()
}
