use colored::Colorize;
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};
use std::{fmt::Display, time::Duration};

#[derive(Copy, Clone, PartialEq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add<(i32, i32)> for Pos {
    type Output = Pos;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Self {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

struct Board {
    tiles: Vec<char>,
    width: usize,
    height: usize,
    player_pos: Pos,
    hunter_pos: Pos,
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

    fn at(&self, pos: Pos) -> Option<&char> {
        let i = self.pos2index(pos);
        self.tiles.get(i)
    }

    fn set_at(&mut self, pos: Pos, c: char) {
        let i = self.pos2index(pos);
        self.tiles[i] = c;
    }

    fn pos2index(&self, pos: Pos) -> usize {
        (pos.y * self.width as i32 + pos.x) as usize
    }
}

fn find_char(tiles: &[char], cc: char, width: usize) -> Pos {
    tiles
        .iter()
        .enumerate()
        .find(|(_, c)| **c == cc)
        .map(|(i, _)| {
            let y = i / width;
            let x = i - y * width;
            Pos::new(x as i32, y as i32)
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
        if poll(Duration::from_millis(100)).unwrap() {
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

enum MoveOutcome {
    Moved,
    NotMoved,
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
        return MoveOutcome::Moved;
    }
    MoveOutcome::NotMoved
}

fn is_dead(board: &Board) -> bool {
    board.player_pos == board.hunter_pos
}

fn main() {
    let mut board = Board::new_test_01();

    loop {
        println!("{board}");

        let key = get_key();
        if key == KeyCode::Esc {
            break;
        }

        let maybe_offset = match key {
            KeyCode::Left => Some((-1, 0)),
            KeyCode::Right => Some((1, 0)),
            KeyCode::Up => Some((0, -1)),
            KeyCode::Down => Some((0, 1)),
            _ => None,
        };

        if let Some(offset) = maybe_offset {
            let outcome = move_player(&mut board, offset);
            if let MoveOutcome::Moved = outcome {
                if is_dead(&board) {
                    break;
                }
            }
        }
    }
    println!("game over");
}
