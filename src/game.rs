use std::io;

use colored::Colorize;
use crossterm::{
    execute,
    terminal::{self, ClearType},
};

use crate::{board::Board, utils::Pos};

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

pub(super) enum GameOutcome {
    Dead,
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
        board.set_at(hunter_pos, ' ');
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
        "Steal the {} from the {}. Use 🡄 🡆 🡅 🡇 to move",
        "$".bright_white(),
        "berserker king".red(),
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
