use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::models::{AppResult, MakeMove};

pub async fn make_move(
    redis: &mut ConnectionManager,
    pool: PgPool,
    room_id: i64,
    session_token: String,
    player_move: MakeMove,
) -> AppResult<()> {
    todo!()
}
