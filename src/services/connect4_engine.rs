use std::collections::HashMap;

const MAX_BOARD_HEIGHT: u8 = 6;

pub struct BoardState<T> {
    // Maps a coordinate (x, y) to a player ID
    positions: HashMap<(u64, u8), T>,

    // Maps an x-coordinate to its occupied height
    heights: HashMap<u64, u8>,
}

pub struct PlayerMove<T> {
    pub player_id: T,
    pub x: u64,
}

struct PlayerPosition<T> {
    player_id: T,
    x: u64,
    y: u8,
}

pub enum Connect4Result {
    WinningMove,
    AllowedMove,
    IllegalMove(String),
}

pub fn make_move<T>(board_state: BoardState<T>, current_move: PlayerMove<T>) -> Connect4Result {
    // Drop token into board
    let current_height = board_state.heights.get(&current_move.x).unwrap_or(&0);

    // Check horizontal

    // Check vertical
    // Check diagonal (top right to bottom left)
    // Check diagonal (top left to bottom right)

    todo!()
}
