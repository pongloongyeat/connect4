use chrono::{DateTime, Utc};
use sqlx::PgConnection;

pub async fn find_player_id(
    connection: &mut PgConnection,
    session_hash: &str,
    room_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT player_id FROM refresh_tokens
        WHERE
            session_hash = $1
            AND room_id = $2
            AND expires_at > $3
            AND revoked = FALSE
        ORDER BY issued_at DESC
        LIMIT 1
    "#,
        session_hash,
        room_id,
        Utc::now().naive_utc()
    )
    .fetch_optional(connection)
    .await
    .map(|r| r.map(|r| r.player_id))?;

    Ok(result)
}

pub async fn create_refresh_token(
    connection: &mut PgConnection,
    token_hash: &str,
    room_id: i64,
    player_id: i64,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO refresh_tokens
        (token_hash, room_id, player_id, issued_at, expires_at)
        VALUES
        ($1, $2, $3, $4, $5)
    "#,
        token_hash,
        room_id,
        player_id,
        Utc::now().naive_utc(),
        expires_at.naive_utc()
    )
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn revoke_all_by_session_hash(
    connection: &mut PgConnection,
    session_hash: &str,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked = TRUE, revoked_at = $1
        WHERE session_hash = $2 AND revoked = FALSE
    "#,
        Utc::now().naive_utc(),
        session_hash
    )
    .execute(connection)
    .await?;

    Ok(())
}
