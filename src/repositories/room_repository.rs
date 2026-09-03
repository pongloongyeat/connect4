use chrono::Utc;
use sqlx::PgConnection;

use crate::models::{CreateRoom, RoomDetails};

pub async fn find_active_room(
    connection: &mut PgConnection,
    id: i64,
) -> Result<Option<RoomDetails>, sqlx::Error> {
    let result = sqlx::query_as!(
        RoomDetails,
        r#"
        SELECT id, display_name, token_hash, is_private, player_count
        FROM rooms
        WHERE id = $1 AND is_active = TRUE
    "#,
        id
    )
    .fetch_optional(connection)
    .await?;

    Ok(result)
}

pub async fn list_rooms_paginated(
    connection: &mut PgConnection,
    offset: i32,
    limit: i32,
) -> Result<Vec<RoomDetails>, sqlx::Error> {
    let result = sqlx::query_as!(
        RoomDetails,
        r#"
        SELECT id, token_hash, is_private, display_name, player_count
        FROM rooms
        WHERE is_active = TRUE
        ORDER BY player_count DESC
        LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64,
    )
    .fetch_all(connection)
    .await?;

    Ok(result)
}

pub async fn create_room(
    connection: &mut PgConnection,
    game: CreateRoom,
) -> Result<RoomDetails, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO rooms
        (created_at, display_name, token_hash, is_private)
        VALUES
        ($1, $2, $3, $4)
        RETURNING id
    "#,
        Utc::now().naive_utc(),
        game.display_name,
        game.token_hash,
        game.is_private,
    )
    .fetch_one(connection)
    .await?;

    Ok(RoomDetails {
        id: result.id,
        token_hash: game.token_hash,
        is_private: game.is_private,
        display_name: game.display_name,
        player_count: 0,
    })
}
