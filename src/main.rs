use colored::Colorize;
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};
use std::{fmt::Display, time::Duration};

const DEAD_MESSAGE: &str = "The berserker king hits you. You die...";
const WIN_MESSAGE: &str = "CoNgRaTs!";

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
    _height: usize,
    player_pos: Pos,
    hunter_pos: Pos,
    exit_pos: Pos,
}

impl Board {
    fn new_test_01() -> Self {
        #[rustfmt::skip]
        let tiles = vec![
                '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
                '#', ' ', ' ', '@', ' ', ' ', ' ', ' ', ' ', ' ', '=', '#',
                '#', ' ', ' ', ' ', '#', '#', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', '#', ' ', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', '#', '#', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '$', ' ', ' ', '#',
                '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
            ];
        let width = 12;
        let player_pos = find_char(&tiles, '@', width);
        let hunter_pos = find_char(&tiles, '=', width);
        let exit_pos = find_char(&tiles, '$', width);

        Self {
            tiles,
            width,
            _height: 8,
            player_pos,
            hunter_pos,
            exit_pos,
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
                        '$' => "$".bright_white(),
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

fn move_hunter_internal(board: &mut Board, offset: (i32, i32)) -> MoveOutcome {
    if offset == (0, 0) {
        return MoveOutcome::NotMoved;
    }
    let dest = board.hunter_pos + offset;
    let at_dest = board.at(dest);
    let Some(at_dest) = at_dest else {
        return MoveOutcome::NotMoved;
    };
    if at_dest != &'#' {
        board.set_at(board.hunter_pos, ' ');
        board.set_at(dest, '=');
        board.hunter_pos = dest;
        return MoveOutcome::Moved;
    }
    MoveOutcome::NotMoved
}

fn move_hunter(board: &mut Board) -> MoveOutcome {
    let dist_x = (board.player_pos.x - board.hunter_pos.x).abs();
    let dist_y = (board.player_pos.y - board.hunter_pos.y).abs();

    let horizontal_chase = match board.player_pos.x.cmp(&board.hunter_pos.x) {
        std::cmp::Ordering::Less => (-1, 0),
        std::cmp::Ordering::Equal => (0, 0),
        std::cmp::Ordering::Greater => (1, 0),
    };

    let vertical_chase = match board.player_pos.y.cmp(&board.hunter_pos.y) {
        std::cmp::Ordering::Less => (0, -1),
        std::cmp::Ordering::Equal => (0, 0),
        std::cmp::Ordering::Greater => (0, 1),
    };

    if dist_x < dist_y {
        let outcome = move_hunter_internal(board, horizontal_chase);
        match outcome {
            MoveOutcome::Moved => MoveOutcome::Moved,
            MoveOutcome::NotMoved => move_hunter_internal(board, vertical_chase),
        }
    } else {
        let outcome = move_hunter_internal(board, vertical_chase);
        match outcome {
            MoveOutcome::Moved => MoveOutcome::Moved,
            MoveOutcome::NotMoved => move_hunter_internal(board, horizontal_chase),
        }
    }
}

fn is_dead(board: &Board) -> bool {
    board.player_pos == board.hunter_pos
}

fn is_win(board: &Board) -> bool {
    board.player_pos == board.exit_pos
}

fn main() {
    let mut board = Board::new_test_01();

    println!("{board}");
    'outer: loop {
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
                    println!("{}", DEAD_MESSAGE.red());
                    break;
                }
                if is_win(&board) {
                    println!("{}", WIN_MESSAGE.green());
                    break;
                }

                for _ in 0..2 {
                    let outcome = move_hunter(&mut board);
                    if let MoveOutcome::Moved = outcome {
                        if is_dead(&board) {
                            println!("{}", DEAD_MESSAGE.red());
                            break 'outer;
                        }
                    }
                }
            }
        }
        println!("{board}");
    }

    println!("game over");
    println!("{board}");
}
