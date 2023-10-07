use std::io;

use colored::Colorize;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{self, ClearType},
};

use crate::{
    board::{Board, BoardTensor},
    utils::Pos,
};

#[derive(PartialEq)]
pub(super) enum MoveOutcome {
    Moved(Pos),
    NotMoved,
}

pub(super) enum TickOutcome {
    Dead,
    Alive(MoveOutcome),
    Victory,
}

pub(super) enum TickOutcomeTensor {
    Continue,
    Victory,
}

pub(super) enum GameOutcome {
    Dead,
    Victory,
    Exit,
}

pub(super) enum GameOutcomeTensor {
    Victory,
    Exit,
}

fn move_player(board: &mut Board, offset: (i32, i32)) -> MoveOutcome {
    let dest = board.player_pos + offset;
    let at_dest = board.at(dest);
    let Some(at_dest) = at_dest else {
        return MoveOutcome::NotMoved;
    };
    if at_dest != &'#' {
        board.set_at(board.player_pos, ' ');
        board.set_at(dest, '@');
        board.player_pos = dest;
        return MoveOutcome::Moved(dest);
    }
    MoveOutcome::NotMoved
}

fn move_hunter_internal(board: &mut Board, offset: (i32, i32), hunter_pos: Pos) -> MoveOutcome {
    if offset == (0, 0) {
        return MoveOutcome::NotMoved;
    }
    let dest = hunter_pos + offset;
    let at_dest = board.at(dest);
    let Some(at_dest) = at_dest else {
        return MoveOutcome::NotMoved;
    };
    if at_dest != &'#' && at_dest != &'K' {
        board.set_at(
            hunter_pos,
            if hunter_pos == board.exit_pos {
                '$'
            } else {
                ' '
            },
        );
        board.set_at(dest, 'K');
        return MoveOutcome::Moved(dest);
    }
    MoveOutcome::NotMoved
}

fn move_hunters(board: &mut Board) -> bool {
    let mut moved = false;
    let new_hunter_pos = board
        .hunters_pos
        .clone()
        .into_iter()
        .map(|hunter| {
            let dist_x = (board.player_pos.x - hunter.x).abs();
            let dist_y = (board.player_pos.y - hunter.y).abs();

            let horizontal_chase = match board.player_pos.x.cmp(&hunter.x) {
                std::cmp::Ordering::Less => (-1, 0),
                std::cmp::Ordering::Equal => (0, 0),
                std::cmp::Ordering::Greater => (1, 0),
            };

            let vertical_chase = match board.player_pos.y.cmp(&hunter.y) {
                std::cmp::Ordering::Less => (0, -1),
                std::cmp::Ordering::Equal => (0, 0),
                std::cmp::Ordering::Greater => (0, 1),
            };

            if dist_x < dist_y {
                let outcome = move_hunter_internal(board, horizontal_chase, hunter);
                match outcome {
                    MoveOutcome::Moved(dest) => {
                        moved = true;
                        dest
                    }
                    MoveOutcome::NotMoved => {
                        match move_hunter_internal(board, vertical_chase, hunter) {
                            MoveOutcome::Moved(dest) => {
                                moved = true;
                                dest
                            }
                            MoveOutcome::NotMoved => hunter,
                        }
                    }
                }
            } else {
                let outcome = move_hunter_internal(board, vertical_chase, hunter);
                match outcome {
                    MoveOutcome::Moved(dest) => {
                        moved = true;
                        dest
                    }
                    MoveOutcome::NotMoved => {
                        match move_hunter_internal(board, horizontal_chase, hunter) {
                            MoveOutcome::Moved(dest) => {
                                moved = true;
                                dest
                            }
                            MoveOutcome::NotMoved => hunter,
                        }
                    }
                }
            }
        })
        .collect();
    board.hunters_pos = new_hunter_pos;
    moved
}

fn is_dead(board: &Board) -> bool {
    board
        .hunters_pos
        .iter()
        .any(|hunter_pos| board.player_pos == *hunter_pos)
}

fn is_win(board: &Board) -> bool {
    board.player_pos == board.exit_pos
}

pub(super) fn print_board(board: &Board) {
    let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
    println!(
        "{} {} {} {}. Use arrow keys to move",
        "Steal the".green(),
        "$".bright_white(),
        "from the".green(),
        "berserker king".red(),
    );
    println!();
    println!("{board}");
}

pub(super) fn print_board_tensor(board: &BoardTensor) {
    let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
    println!(
        "{} {} {}",
        board.amygdala_count.to_string().yellow(),
        if board.amygdala_count == 1 {
            "amygdala".white()
        } else {
            "amygdalas".white()
        },
        "left, keep rotating!".green()
    );
    println!();
    println!("{board}");
}

pub(super) fn tick(board: &mut Board, offset: (i32, i32)) -> TickOutcome {
    let outcome = move_player(board, offset);
    if let MoveOutcome::Moved(_) = outcome {
        if is_dead(&*board) {
            return TickOutcome::Dead;
        }
        if is_win(&*board) {
            return TickOutcome::Victory;
        }

        for _ in 0..2 {
            let outcome = move_hunters(board);
            if outcome && is_dead(&*board) {
                return TickOutcome::Dead;
            }
        }
        return TickOutcome::Alive(MoveOutcome::Moved(Pos::new(0, 0)));
    }
    TickOutcome::Alive(MoveOutcome::NotMoved)
}

fn gravity(board: &mut BoardTensor) {
    let mut moved = true;
    while moved {
        moved = false;
        for y in (0..12).rev() {
            for x in (0..12).rev() {
                let pos = Pos::new(x, y);
                match board.at(pos) {
                    Some(1) | Some(2) => {
                        let new_pos = Pos::new(pos.x, pos.y + 1);
                        let c = *board.at(pos).unwrap();
                        if let Some(0) = board.at(new_pos) {
                            moved = true;
                            board.set_at(pos, 0);
                            board.set_at(new_pos, c);
                            if c == 1 {
                                board.player_pos = new_pos;
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

pub(super) fn tick_tensor(board: &mut BoardTensor, key: KeyCode) -> TickOutcomeTensor {
    let outcome = interpret_key(key, board);
    gravity(board);
    outcome
}

fn interpret_key(key: KeyCode, board: &mut BoardTensor) -> TickOutcomeTensor {
    match key {
        KeyCode::Left => {
            let pos = board.player_pos;
            let new_pos = Pos::new(pos.x - 1, pos.y);
            match board.at(new_pos) {
                Some(0) => {
                    board.set_at(pos, 0);
                    board.set_at(new_pos, 1);
                    board.player_pos = new_pos;
                }
                Some(2) => {
                    board.set_at(pos, 0);
                    board.set_at(new_pos, 1);
                    board.player_pos = new_pos;
                    board.amygdala_count -= 1;
                    if board.amygdala_count == 0 {
                        return TickOutcomeTensor::Victory;
                    }
                }
                _ => return TickOutcomeTensor::Continue,
            }
        }
        KeyCode::Right => {
            let pos = board.player_pos;
            let new_pos = Pos::new(pos.x + 1, pos.y);
            match board.at(new_pos) {
                Some(0) => {
                    board.set_at(pos, 0);
                    board.set_at(new_pos, 1);
                    board.player_pos = new_pos;
                }
                Some(2) => {
                    board.set_at(pos, 0);
                    board.set_at(new_pos, 1);
                    board.player_pos = new_pos;
                    board.amygdala_count -= 1;
                    if board.amygdala_count == 0 {
                        return TickOutcomeTensor::Victory;
                    }
                }
                _ => return TickOutcomeTensor::Continue,
            }
        }
        KeyCode::Up => {
            rotate_right(board);
        }
        KeyCode::Down => {
            rotate_left(board);
        }
        _ => return TickOutcomeTensor::Continue,
    }
    return TickOutcomeTensor::Continue;
}

fn rotate_right(board: &mut BoardTensor) {
    let mut new_board = board.clone();
    for y in 0..12 {
        for x in 0..12 {
            let c = board.at(Pos::new(x, y)).unwrap();
            let new_pos = Pos::new(11 - y, x);
            if *c == 1 {
                new_board.player_pos = new_pos;
            }
            new_board.set_at(new_pos, *c)
        }
    }
    *board = new_board;
}

fn rotate_left(board: &mut BoardTensor) {
    // :-)))
    rotate_right(board);
    rotate_right(board);
    rotate_right(board);
}
