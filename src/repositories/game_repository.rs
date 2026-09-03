use chrono::Utc;
use sqlx::PgConnection;

use crate::models::{CreateGame, GameDetails};

pub async fn find_active_game(
    connection: &mut PgConnection,
    id: i64,
) -> Result<Option<GameDetails>, sqlx::Error> {
    let result = sqlx::query_as!(
        GameDetails,
        r#"
        SELECT id, display_name, token_hash, is_private, player_count
        FROM games
        WHERE id = $1 AND is_active = TRUE
    "#,
        id
    )
    .fetch_optional(connection)
    .await?;

    Ok(result)
}

pub async fn list_games_paginated(
    connection: &mut PgConnection,
    offset: i32,
    limit: i32,
) -> Result<Vec<GameDetails>, sqlx::Error> {
    let result = sqlx::query_as!(
        GameDetails,
        r#"
        SELECT id, token_hash, is_private, display_name, player_count
        FROM games
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

pub async fn create_game(
    connection: &mut PgConnection,
    game: CreateGame,
) -> Result<GameDetails, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO games
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

    Ok(GameDetails {
        id: result.id,
        token_hash: game.token_hash,
        is_private: game.is_private,
        display_name: game.display_name,
        player_count: 0,
    })
}
