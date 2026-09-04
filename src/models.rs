use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::Error as DeError, ser::SerializeMap};

pub struct RoomDetails {
    pub id: i64,
    pub invite_code_hash: String,
    pub player_count: i64,
}

pub struct CreateRoom {
    pub invite_code_hash: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomResponse {
    pub id: i64,
    pub invite_code: String,
}

pub struct CurrentPlayer {
    pub id: i64,
    pub ref_no: i64,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentPlayerResponse {
    pub id: i64,
    pub display_name: Option<String>,
    pub access_token: String,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token: String,
    pub refresh_token_expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentRoomResponse {
    pub id: i64,
    pub player_count: i64,
    pub current_player: CurrentPlayerResponse,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshSessionRequest {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeMoveRequest {
    pub column: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MakeMove {
    pub room_id: i64,
    pub player_id: i64,
    pub column: u64,
}

#[derive(Clone)]
pub struct BoardState<T> {
    // Maps a coordinate (x, y) to a player ID
    pub positions: HashMap<(u64, u8), T>,

    // Maps an x-coordinate to its occupied height
    pub heights: HashMap<u64, u8>,
}

const COORD_DELIMITER: &'static str = ",";

impl<T> Serialize for BoardState<T>
where
    T: Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        let positions = self
            .positions
            .clone()
            .into_iter()
            .map(|((x, y), v)| (format!("{x}{COORD_DELIMITER}{y}"), v))
            .collect::<HashMap<String, T>>();

        map.serialize_entry("positions", &positions)?;
        map.serialize_entry("heights", &self.heights)?;

        map.end()
    }
}

impl<'de, T> Deserialize<'de> for BoardState<T>
where
    T: Deserialize<'de> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StoredBoardState<T> {
            positions: HashMap<String, T>,
            heights: HashMap<u64, u8>,
        }

        let stored = StoredBoardState::<T>::deserialize(deserializer)?;
        let positions = stored
            .positions
            .clone()
            .iter()
            .map(|(coords, v)| {
                let (x, y) = coords.split_once(COORD_DELIMITER).ok_or_else(|| {
                    tracing::error!("Failed to split coordinate: {coords}");
                    D::Error::custom("invalid coordinate")
                })?;

                let x = x
                    .parse::<u64>()
                    .inspect_err(|err| tracing::error!("Invalid value for coordinate: {x}, {err}"))
                    .map_err(|_| D::Error::custom("invalid x coordinate"))?;
                let y = y
                    .parse::<u8>()
                    .inspect_err(|err| tracing::error!("Invalid value for coordinate: {y}, {err}"))
                    .map_err(|_| D::Error::custom("invalid y coordinate"))?;

                Ok(((x, y), v.to_owned()))
            })
            .collect::<Result<HashMap<_, _>, D::Error>>()?;

        Ok(BoardState {
            positions,
            heights: stored.heights,
        })
    }
}
