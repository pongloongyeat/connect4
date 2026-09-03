use sqlx::PgPool;

use crate::{
    models::{
        AppError, AppResult, CreateRoom, CreateRoomRequest, CreateRoomResponse, RoomListingResponse,
    },
    repositories::room_repository,
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
