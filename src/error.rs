use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

pub enum AppError {
    DatabaseError(sqlx::Error),
    RedisError(redis::RedisError),

    InternalError(Option<&'static str>),

    RoomDoesNotExist,
    MissingInviteCode,
    InvalidInviteCode,
    PlayerDoesNotExist,

    IllegalMove(&'static str),
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(value: redis::RedisError) -> Self {
        Self::RedisError(value)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    #[serde(skip)]
    status_code: StatusCode,
    code: String,
    message: String,
}

const DEFAULT_ERROR_MESSAGE: &'static str = "An unknown error has occured.";

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        let (status_code, code, message) = match value {
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "999.999",
                DEFAULT_ERROR_MESSAGE,
            ),
            AppError::RedisError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "999.998",
                DEFAULT_ERROR_MESSAGE,
            ),
            AppError::InternalError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "999.001",
                error.unwrap_or(DEFAULT_ERROR_MESSAGE),
            ),
            AppError::RoomDoesNotExist => {
                (StatusCode::NOT_FOUND, "100.001", "Room does not exist.")
            }
            AppError::MissingInviteCode => {
                (StatusCode::BAD_REQUEST, "100.002", "Missing invite code.")
            }
            AppError::InvalidInviteCode => {
                (StatusCode::BAD_REQUEST, "100.003", "Invalid invite code.")
            }
            AppError::PlayerDoesNotExist => {
                (StatusCode::NOT_FOUND, "101.001", "Player does not exist.")
            }
            AppError::IllegalMove(err) => (StatusCode::BAD_REQUEST, "102.001", err),
        };

        Self {
            status_code,
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, Json(self)).into_response()
    }
}
