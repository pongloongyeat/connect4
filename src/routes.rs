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
        ApiError, ApiResult, CreateRoomRequest, CreateRoomResponse, CurrentRoomResponse,
        JoinRoomRequest, RoomListingResponse, SessionToken,
    },
    services::{player_service, room_service},
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/rooms", post(create_room))
        .route("/rooms", get(list_rooms))
        .route("/rooms/{id}/join", post(join_room))
        .with_state(state)
}

#[axum::debug_handler]
async fn create_room(
    State(state): State<AppState>,
    Json(request): Json<CreateRoomRequest>,
) -> ApiResult<(StatusCode, Json<CreateRoomResponse>)> {
    let response = room_service::create_room(state.pool, request)
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
async fn list_rooms(
    State(state): State<AppState>,
    Query(query): Query<ListGamesQuery>,
) -> ApiResult<Json<Vec<RoomListingResponse>>> {
    let response = room_service::list_rooms(state.pool, query.offset, query.limit)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(response))
}

#[axum::debug_handler]
async fn join_room(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    TypedHeader(SessionToken(token)): TypedHeader<SessionToken>,
    Json(request): Json<JoinRoomRequest>,
) -> ApiResult<Json<CurrentRoomResponse>> {
    let response = player_service::join_room(state.pool, id, token, request)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(response))
}
