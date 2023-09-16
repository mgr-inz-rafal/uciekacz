use colored::Colorize;
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};
use std::{fmt::Display, time::Duration};

struct Board {
    tiles: Vec<char>,
    width: usize,
    height: usize,
    player_pos: (usize, usize),
    hunter_pos: (usize, usize),
}

impl Board {
    fn new_test_01() -> Self {
        #[rustfmt::skip]
        let tiles = vec![
                '#', '#', '#', '#', '#', '#', '#',
                '#', ' ', ' ', '#', ' ', '=', '#',
                '#', ' ', '@', '#', ' ', ' ', '#',
                '#', ' ', '#', '#', ' ', ' ', '#',
                '#', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', '#', ' ', '#',
                '#', ' ', ' ', ' ', '#', ' ', '#',
                '#', '#', '#', '#', '#', '#', '#',
            ];
        let width = 7;
        let player_pos = find_char(&tiles, '@', width);
        let hunter_pos = find_char(&tiles, '=', width);

        Self {
            tiles,
            width,
            height: 8,
            player_pos,
            hunter_pos,
        }
    }
}

fn find_char(tiles: &[char], cc: char, width: usize) -> (usize, usize) {
    tiles
        .iter()
        .enumerate()
        .find(|(_, c)| **c == cc)
        .map(|(i, _)| {
            let y = i / width;
            let x = i - y * width;
            (x, y)
        })
        .expect("no {c}")
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chunks = self.tiles.chunks(self.width);
        for chunk in chunks {
            for c in chunk {
                write!(
                    f,
                    "{}",
                    match c {
                        '#' => "#".blue(),
                        '@' => "@".bright_green(),
                        '=' => "K".red(),
                        _ => " ".black(),
                    }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn get_key() -> KeyCode {
    loop {
        if poll(Duration::from_millis(1_000)).unwrap() {
            let event = read().unwrap();
            match event {
                Event::Key(ev) if ev.kind == KeyEventKind::Press => {
                    return ev.code;
                }
                _ => (),
            }
        }
    }
}

fn main() {
    let b = Board::new_test_01();

    loop {
        println!("{b}");

        let key = get_key();
        if key == KeyCode::Esc {
            break;
        }
    }
    println!("game over");
}
