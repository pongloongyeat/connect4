use chrono::Utc;
use sqlx::PgConnection;

use crate::models::CurrentPlayer;

pub async fn find_player(
    connection: &mut PgConnection,
    room_id: i64,
    session_hash: String,
) -> Result<Option<CurrentPlayer>, sqlx::Error> {
    let result = sqlx::query_as!(
        CurrentPlayer,
        r#"
        SELECT id, display_name
        FROM players
        WHERE room_id = $1 AND session_hash = $2
    "#,
        room_id,
        session_hash
    )
    .fetch_optional(connection)
    .await?;

    Ok(result)
}

pub async fn create_player(
    connection: &mut PgConnection,
    room_id: i64,
    session_hash: String,
) -> Result<CurrentPlayer, sqlx::Error> {
    let result = sqlx::query_as!(
        CurrentPlayer,
        r#"
        INSERT INTO players
        (room_id, session_hash, joined_at)
        VALUES
        ($1, $2, $3)
        ON CONFLICT DO NOTHING
        RETURNING id, display_name
    "#,
        room_id,
        session_hash,
        Utc::now().naive_utc()
    )
    .fetch_one(connection)
    .await?;

    Ok(result)
}
