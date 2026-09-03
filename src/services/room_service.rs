use sqlx::{Connection, PgPool};

use crate::{
    models::{
        AppError, AppResult, CreateRoom, CreateRoomRequest, CreateRoomResponse,
        CurrentPlayerResponse, CurrentRoomResponse, JoinRoomRequest, RoomListingResponse,
    },
    repositories::{player_repository, room_repository},
    services::DEFAULT_LIMIT,
    utils,
};

const INVITE_CODE_LENGTH: u8 = 6;

pub async fn create_room(pool: PgPool, room: CreateRoomRequest) -> AppResult<CreateRoomResponse> {
    let invite_code = if room.is_private {
        Some(utils::generate_invite_code(INVITE_CODE_LENGTH))
    } else {
        None
    };
    let token_hash = invite_code.as_ref().map(|code| utils::sha256_hash(&code));

    let display_name = room.display_name;
    let room = CreateRoom {
        display_name: display_name.clone(),
        token_hash: token_hash,
        is_private: room.is_private,
    };

    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let created_room = room_repository::create_room(&mut connection, room)
        .await
        .map_err(AppError::from)?;

    Ok(CreateRoomResponse {
        id: created_room.id,
        display_name: display_name,
        invite_code: invite_code,
    })
}

pub async fn list_rooms(
    pool: PgPool,
    offset: Option<i32>,
    limit: Option<i32>,
) -> AppResult<Vec<RoomListingResponse>> {
    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let rooms = room_repository::list_rooms_paginated(&mut connection, offset, limit)
        .await
        .map_err(AppError::from)?;

    Ok(rooms
        .iter()
        .map(|v| RoomListingResponse {
            id: v.id,
            display_name: v.display_name.clone(),
            player_count: v.player_count,
        })
        .collect())
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
