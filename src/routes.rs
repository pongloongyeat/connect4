use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use axum_extra::TypedHeader;

use crate::{
    models::{ApiError, ApiResult, CreateRoomResponse, CurrentRoomResponse, SessionToken},
    services::room_service,
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/rooms", post(create_room))
        .route("/rooms/{invite_code}/join", post(join_room))
        .with_state(state)
}

#[axum::debug_handler]
async fn create_room(
    State(state): State<AppState>,
) -> ApiResult<(StatusCode, Json<CreateRoomResponse>)> {
    let response = room_service::create_room(&state.pool)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[axum::debug_handler]
async fn join_room(
    State(state): State<AppState>,
    Path(invite_code): Path<String>,
    TypedHeader(SessionToken(token)): TypedHeader<SessionToken>,
) -> ApiResult<Json<CurrentRoomResponse>> {
    let response = room_service::join_room(&state.pool, &token, &"TOKEN", &invite_code)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(response))
}
