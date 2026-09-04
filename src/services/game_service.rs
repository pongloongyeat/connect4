use sqlx::{PgConnection, PgPool};

use crate::{
    error::{AppError, AppResult},
    models::{BoardState, CurrentPlayer, MakeMove, MakeMoveRequest},
    repositories::{player_repository, room_repository},
    services::connect4_engine::{self, PlayerMove},
};

async fn get_board_state(
    connection: &mut PgConnection,
    room_id: i64,
) -> AppResult<BoardState<i64>> {
    let game_state_json = room_repository::get_game_state(connection, room_id)
        .await
        .map_err(AppError::from)?;

    let Some(game_state_json) = game_state_json else {
        return Err(AppError::RoomDoesNotExist);
    };

    serde_json::from_value::<BoardState<i64>>(game_state_json)
        .inspect_err(|err| tracing::error!("Failed to deserialize board state {room_id}: {err}"))
        .map_err(|_| AppError::InternalError(Some("Failed to deserialize board state.")))
}

pub async fn make_move(
    pool: &PgPool,
    room_id: i64,
    player_id: i64,
    current_move: &MakeMoveRequest,
) -> AppResult<()> {
    let current_move = MakeMove {
        room_id,
        player_id,
        column: current_move.column,
    };
    // TODO: Move entire thing to Kafka

    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let board_state = get_board_state(&mut connection, room_id).await?;

    let player_move = PlayerMove {
        player_id,
        x: current_move.column,
    };
    let (result, board_state) = connect4_engine::make_move(board_state, player_move);
    match result {
        connect4_engine::Connect4Result::IllegalMove(err) => {
            return Err(AppError::IllegalMove(err));
        }
        connect4_engine::Connect4Result::AllowedMove => update_board_state(&board_state).await?,
        connect4_engine::Connect4Result::WinningMove => {
            let _ = update_board_state(&board_state).await?;
            let _ = end_game(pool, room_id, player_id, &board_state).await?;
        }
    };

    Ok(())
}

async fn update_board_state(board_state: &BoardState<i64>) -> AppResult<()> {
    todo!()
}

async fn end_game(
    pool: &PgPool,
    room_id: i64,
    winning_player_id: i64,
    board_state: &BoardState<i64>,
) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;
    let rows_affected = room_repository::end_game(&mut tx, room_id)
        .await
        .map_err(AppError::from)?;
    if rows_affected != 1 {
        tracing::error!("Attempted to end a non-existing room {room_id}");
        return Err(AppError::InternalError(Some("Room does not exist")));
    }

    let rows_affected = player_repository::set_winning_player(&mut tx, winning_player_id)
        .await
        .map_err(AppError::from)?;
    if rows_affected != 1 {
        tracing::error!(
            "Attempted to set winning player to non existent player {winning_player_id}"
        );
        return Err(AppError::InternalError(Some("Player does not exist")));
    }
    tx.commit().await.map_err(AppError::from)?;

    let mut connection = pool.acquire().await.map_err(AppError::from)?;
    let player = player_repository::find_player_by_id(&mut connection, winning_player_id)
        .await
        .map_err(AppError::from)?;

    let _ = push_update(board_state, player).await?;

    Ok(())
}

async fn push_update(
    board_state: &BoardState<i64>,
    winning_player: Option<CurrentPlayer>,
) -> AppResult<()> {
    todo!()
}
