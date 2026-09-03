use sqlx::PgPool;

use crate::{
    models::{
        AppError, AppResult, CreateGame, CreateGameRequest, CreateGameResponse, GameListingResponse,
    },
    repositories::game_repository,
    services::DEFAULT_LIMIT,
    utils,
};

const INVITE_CODE_LENGTH: u8 = 6;

pub async fn create_game(pool: PgPool, game: CreateGameRequest) -> AppResult<CreateGameResponse> {
    let invite_code = if game.is_private {
        Some(utils::generate_invite_code(INVITE_CODE_LENGTH))
    } else {
        None
    };
    let token_hash = invite_code.as_ref().map(|code| utils::sha256_hash(&code));

    let display_name = game.display_name;
    let game = CreateGame {
        display_name: display_name.clone(),
        token_hash: token_hash,
        is_private: game.is_private,
    };

    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let created_game = game_repository::create_game(&mut connection, game)
        .await
        .map_err(AppError::from)?;

    Ok(CreateGameResponse {
        id: created_game.id,
        display_name: display_name,
        invite_code: invite_code,
    })
}

pub async fn list_games(
    pool: PgPool,
    offset: Option<i32>,
    limit: Option<i32>,
) -> AppResult<Vec<GameListingResponse>> {
    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let games = game_repository::list_games_paginated(&mut connection, offset, limit)
        .await
        .map_err(AppError::from)?;

    Ok(games
        .iter()
        .map(|v| GameListingResponse {
            id: v.id,
            display_name: v.display_name.clone(),
            player_count: v.player_count,
        })
        .collect())
}
