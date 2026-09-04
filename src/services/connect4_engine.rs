use std::collections::HashMap;

use crate::models::BoardState;

const MAX_BOARD_HEIGHT: u8 = 6;
const CONSECUTIVE_FOR_WIN: u8 = 4;

pub struct PlayerMove<T> {
    pub player_id: T,
    pub x: u64,
}

pub enum Connect4Result {
    WinningMove,
    AllowedMove,
    IllegalMove(&'static str),
}

pub fn make_move<T>(
    board_state: BoardState<T>,
    current_move: PlayerMove<T>,
) -> (Connect4Result, BoardState<T>)
where
    T: PartialEq + Clone,
{
    // Drop token into board
    let BoardState { positions, heights } = board_state.clone();
    let PlayerMove { player_id, x } = current_move;
    let current_height = heights.get(&x).unwrap_or(&0).to_owned();
    let y = current_height;

    if y + 1 > MAX_BOARD_HEIGHT {
        return (
            Connect4Result::IllegalMove("Exceeded board height"),
            board_state,
        );
    }

    // Insert into board
    let mut positions = positions;
    positions.insert((x, y), player_id.clone());
    let mut heights = heights;
    heights.insert(x, y + 1);

    let poi1 = vec![
        get_position(&positions, &player_id, (x, y), (-2, 0)),
        get_position(&positions, &player_id, (x, y), (-1, 0)),
        get_position(&positions, &player_id, (x, y), (1, 0)),
        get_position(&positions, &player_id, (x, y), (2, 0)),
    ];
    let poi2 = vec![
        get_position(&positions, &player_id, (x, y), (0, -2)),
        get_position(&positions, &player_id, (x, y), (0, -1)),
        get_position(&positions, &player_id, (x, y), (0, 1)),
        get_position(&positions, &player_id, (x, y), (0, 2)),
    ];
    let poi3 = vec![
        get_position(&positions, &player_id, (x, y), (1, 1)),
        get_position(&positions, &player_id, (x, y), (2, 2)),
        get_position(&positions, &player_id, (x, y), (-1, -1)),
        get_position(&positions, &player_id, (x, y), (-2, -2)),
    ];
    let poi4 = vec![
        get_position(&positions, &player_id, (x, y), (-1, 1)),
        get_position(&positions, &player_id, (x, y), (-2, 2)),
        get_position(&positions, &player_id, (x, y), (1, -1)),
        get_position(&positions, &player_id, (x, y), (2, -2)),
    ];

    let result = if vec![poi1, poi2, poi3, poi4]
        .iter()
        .any(|poi| has_consecutives(poi, CONSECUTIVE_FOR_WIN as usize))
    {
        Connect4Result::WinningMove
    } else {
        Connect4Result::AllowedMove
    };

    let updated_board_state = BoardState {
        positions: positions,
        heights: heights,
    };
    (result, updated_board_state)
}

fn get_position<T>(
    positions: &HashMap<(u64, u8), T>,
    player_id: &T,
    coords: (u64, u8),
    offset: (i8, i8),
) -> Option<T>
where
    T: PartialEq + Clone,
{
    let (x, y) = coords;
    let (offset_x, offset_y) = offset;
    let (x, y) = (
        x.checked_add_signed(offset_x as i64),
        y.checked_add_signed(offset_y),
    );

    let Some(x) = x else {
        return None;
    };
    let Some(y) = y else {
        return None;
    };

    let position = positions.get(&(x, y));
    match position {
        Some(position) => {
            if position == player_id {
                Some(position.clone())
            } else {
                None
            }
        }
        None => None,
    }
}

fn has_consecutives<T>(vec: &Vec<T>, target: usize) -> bool
where
    T: PartialEq,
{
    let mut previous: Option<&T> = None;
    let mut count = 0;

    for v in vec {
        if count == target {
            return true;
        }

        if previous.is_none() {
            previous = Some(v);
            continue;
        }

        let current = Some(v);
        if previous == current {
            count += 1;
        } else {
            count = 0;
        }

        previous = current;
    }

    false
}
