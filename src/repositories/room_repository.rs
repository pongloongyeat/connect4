use chrono::Utc;
use sqlx::PgConnection;

use crate::models::{CreateRoom, RoomDetails};

pub async fn find_active_room(
    connection: &mut PgConnection,
    invite_code_hash: &str,
) -> Result<Option<RoomDetails>, sqlx::Error> {
    let result = sqlx::query_as!(
        RoomDetails,
        r#"
        SELECT id, invite_code_hash, player_count
        FROM rooms
        WHERE invite_code_hash = $1 AND active = TRUE
    "#,
        invite_code_hash
    )
    .fetch_optional(connection)
    .await?;

    Ok(result)
}

pub async fn get_game_state(
    connection: &mut PgConnection,
    room_id: i64,
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT game_state FROM rooms
        WHERE id = $1 AND active = TRUE
    "#,
        room_id
    )
    .fetch_optional(connection)
    .await?;

    Ok(result.map(|r| r.game_state))
}

pub async fn create_room(
    connection: &mut PgConnection,
    room: &CreateRoom,
) -> Result<RoomDetails, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO rooms
        (created_at, invite_code_hash)
        VALUES
        ($1, $2)
        RETURNING id AS "id!"
    "#,
        Utc::now().naive_utc(),
        room.invite_code_hash,
    )
    .fetch_one(connection)
    .await?;

    Ok(RoomDetails {
        id: result.id,
        invite_code_hash: room.invite_code_hash.to_string(),
        player_count: 0,
    })
}

pub async fn end_game(connection: &mut PgConnection, room_id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE rooms
        SET active = FALSE, ended_at = $2
        WHERE id = $1
    "#,
        room_id,
        Utc::now().naive_utc()
    )
    .execute(connection)
    .await?;

    Ok(result.rows_affected())
}
