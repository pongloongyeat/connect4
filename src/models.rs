use axum::{
    Json,
    http::{HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
};
use axum_extra::headers::Header;
use serde::{Deserialize, Serialize};

pub type AppResult<T> = Result<T, AppError>;

pub enum AppError {
    DatabaseError(sqlx::Error),
    RedisError(redis::RedisError),

    InternalError(Option<&'static str>),

    GameDoesNotExist,
    MissingInviteCode,
    InvalidInviteCode,
    PlayerDoesNotExist,
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
            AppError::GameDoesNotExist => {
                (StatusCode::NOT_FOUND, "100.001", "Game does not exist.")
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

pub struct SessionToken(pub String);

impl Header for SessionToken {
    fn name() -> &'static axum::http::HeaderName {
        static NAME: HeaderName = HeaderName::from_static("x-session-token");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(axum_extra::headers::Error::invalid)?;

        let token = value
            .to_str()
            .map_err(|_| axum_extra::headers::Error::invalid())?;

        Ok(SessionToken(token.to_string()))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        let value =
            HeaderValue::try_from(&self.0).expect("SessionToken contains an invalid header value");

        values.extend(std::iter::once(value));
    }
}

pub struct GameDetails {
    pub id: i64,
    pub display_name: Option<String>,
    pub token_hash: Option<String>,
    pub is_private: bool,
    pub player_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameListingResponse {
    pub id: i64,
    pub display_name: Option<String>,
    pub player_count: i64,
}

pub struct CreateGame {
    pub display_name: Option<String>,
    pub token_hash: Option<String>,
    pub is_private: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameRequest {
    pub display_name: Option<String>,
    #[serde(rename = "private")]
    pub is_private: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameResponse {
    pub id: i64,
    pub display_name: Option<String>,
    pub invite_code: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinGameRequest {
    pub invite_code: Option<String>,
}

pub struct CurrentPlayer {
    pub id: i64,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentPlayerResponse {
    pub id: i64,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentGameResponse {
    pub id: i64,
    pub display_name: Option<String>,
    pub player_count: i64,
    pub current_player: CurrentPlayerResponse,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeMove {
    column: u64,
}
