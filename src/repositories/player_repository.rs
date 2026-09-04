use chrono::Utc;
use sqlx::PgConnection;

use crate::models::CurrentPlayer;

pub async fn find_player_by_id(
    connection: &mut PgConnection,
    id: i64,
) -> Result<Option<CurrentPlayer>, sqlx::Error> {
    let result = sqlx::query_as!(
        CurrentPlayer,
        r#"
        SELECT id, ref_no, display_name
        FROM players
        WHERE id = $1
    "#,
        id,
    )
    .fetch_optional(connection)
    .await?;

    Ok(result)
}

pub async fn player_ref_no(connection: &mut PgConnection) -> Result<i64, sqlx::Error> {
    let sequence = sqlx::query!(r#"SELECT nextval('players_ref_no_seq') AS "sequence!""#)
        .fetch_one(connection)
        .await?;

    Ok(sequence.sequence)
}

pub async fn create_player(
    connection: &mut PgConnection,
    room_id: i64,
    ref_no: i64,
) -> Result<CurrentPlayer, sqlx::Error> {
    let result = sqlx::query_as!(
        CurrentPlayer,
        r#"
        INSERT INTO players
        (room_id, ref_no, joined_at)
        VALUES
        ($1, $2, $3)
        RETURNING id, ref_no, display_name
    "#,
        room_id,
        ref_no,
        Utc::now().naive_utc()
    )
    .fetch_one(connection)
    .await?;

    Ok(result)
}

pub async fn set_winning_player(
    connection: &mut PgConnection,
    player_id: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE players
        SET winner = TRUE
        WHERE id = $1
    "#,
        player_id
    )
    .execute(connection)
    .await?;

    Ok(result.rows_affected())
}
