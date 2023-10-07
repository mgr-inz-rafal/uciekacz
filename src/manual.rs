use std::io;

use colored::Colorize;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{self, ClearType},
};

use crate::{
    board::{Board, BoardTensor},
    game::{
        print_board, print_board_tensor, tick, tick_tensor, GameOutcome, GameOutcomeTensor,
        TickOutcome,
    },
    utils::get_key,
};

const DEAD_MESSAGE: &str = "The berserker king hits you. You die...";
const WIN_MESSAGE: &str = "CoNgRaTs!";
const BYE_MESSAGE: &str = "Bye!";

pub(super) fn manual_play(mut board: Board) {
    print_board(&board);

    let game_outcome = loop {
        let key = get_key();
        if key == KeyCode::Esc {
            break GameOutcome::Exit;
        }

        let maybe_offset = match key {
            KeyCode::Left => Some((-1, 0)),
            KeyCode::Right => Some((1, 0)),
            KeyCode::Up => Some((0, -1)),
            KeyCode::Down => Some((0, 1)),
            _ => None,
        };

        if let Some(offset) = maybe_offset {
            match tick(&mut board, offset) {
                TickOutcome::Dead => break GameOutcome::Dead,
                TickOutcome::Alive(_) => (),
                TickOutcome::Victory => break GameOutcome::Victory,
            }
        }
        print_board(&board);
    };

    let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
    match game_outcome {
        GameOutcome::Dead => println!("{}", DEAD_MESSAGE.red()),
        GameOutcome::Victory => println!("{}", WIN_MESSAGE.green()),
        GameOutcome::Exit => println!("{}", BYE_MESSAGE.magenta()),
    }
    println!("game over");
    println!("{board}");
}

pub(super) fn manual_play_tensor(mut board: BoardTensor) {
    print_board_tensor(&board);

    let game_outcome = loop {
        let key = get_key();
        if key == KeyCode::Esc {
            break GameOutcomeTensor::Exit;
        }

        match tick_tensor(&mut board, key) {
            crate::game::TickOutcomeTensor::Continue => (),
            crate::game::TickOutcomeTensor::Victory => break GameOutcomeTensor::Victory,
        }

        // if let Some(offset) = maybe_offset {
        //     match tick(&mut board, offset) {
        //         TickOutcome::Dead => break GameOutcome::Dead,
        //         TickOutcome::Alive(_) => (),
        //         TickOutcome::Victory => break GameOutcome::Victory,
        //     }
        // }
        print_board_tensor(&board);
    };

    let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
    match game_outcome {
        GameOutcomeTensor::Victory => println!("{}", WIN_MESSAGE.green()),
        GameOutcomeTensor::Exit => println!("{}", BYE_MESSAGE.magenta()),
    }
    println!("game over");
    println!("{board}");
}
