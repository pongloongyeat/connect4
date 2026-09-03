use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use axum_extra::TypedHeader;
use serde::Deserialize;

use crate::{
    models::{
        ApiError, ApiResult, CreateGameRequest, CreateGameResponse, CurrentGameResponse,
        GameListingResponse, JoinGameRequest, SessionToken,
    },
    services::{game_service, player_service},
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/games", post(create_game))
        .route("/games", get(list_games))
        .route("/games/{id}/join", post(join_game))
        .with_state(state)
}

#[axum::debug_handler]
async fn create_game(
    State(state): State<AppState>,
    Json(request): Json<CreateGameRequest>,
) -> ApiResult<(StatusCode, Json<CreateGameResponse>)> {
    let response = game_service::create_game(state.pool, request)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[derive(Clone, Copy, Deserialize)]
struct ListGamesQuery {
    offset: Option<i32>,
    limit: Option<i32>,
}

#[axum::debug_handler]
async fn list_games(
    State(state): State<AppState>,
    Query(query): Query<ListGamesQuery>,
) -> ApiResult<Json<Vec<GameListingResponse>>> {
    let response = game_service::list_games(state.pool, query.offset, query.limit)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(response))
}

#[axum::debug_handler]
async fn join_game(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    TypedHeader(SessionToken(token)): TypedHeader<SessionToken>,
    Json(request): Json<JoinGameRequest>,
) -> ApiResult<Json<CurrentGameResponse>> {
    let response = player_service::join_game(state.pool, id, token, request)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(response))
}
