use colored::Colorize;
use std::fmt::Display;

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

fn find_char(tiles: &Vec<char>, cc: char, width: usize) -> (usize, usize) {
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
        Ok(self.tiles.chunks(self.width).for_each(|chunk| {
            chunk.into_iter().for_each(|c| {
                print!(
                    "{}",
                    match c {
                        '#' => "#".blue(),
                        '@' => "@".bright_green(),
                        '=' => "K".red(),
                        _ => " ".black(),
                    }
                )
            });
            println!();
        }))
    }
}

fn main() {
    let b = Board::new_test_01();
    println!("{b}");
}
