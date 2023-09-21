use clap::Parser;
use colored::Colorize;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, ClearType},
};
use petgraph::{algo::astar, data::Build, graph::Node, stable_graph::NodeIndex, Graph};
use std::{collections::BTreeSet, fmt::Display, io, ops::ControlFlow, time::Duration};

const DEAD_MESSAGE: &str = "The berserker king hits you. You die...";
const WIN_MESSAGE: &str = "CoNgRaTs!";
const BYE_MESSAGE: &str = "Bye!";

#[derive(Parser, Debug)]
struct Args {
    /// If specified, the game will find the solution automatically.
    #[arg(short, long)]
    auto: bool,
}

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

#[derive(Clone, PartialEq)]
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

        // #[rustfmt::skip]
        // let tiles = vec![
        //         '#', '#', '#', '#',
        //         '#', ' ', '@', '#',
        //         '#', ' ', ' ', '#',
        //         '#', ' ', '$', '#',
        //         '#', ' ', ' ', '#',
        //         '#', ' ', ' ', '#',
        //         '#', '=', ' ', '#',
        //         '#', '#', '#', '#'
        //     ];
        // let width = 4;
        // let player_pos = find_char(&tiles, '@', width);
        // let hunter_pos = find_char(&tiles, '=', width);
        // let exit_pos = find_char(&tiles, '$', width);

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
    // return KeyCode::BackTab;

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

fn get_key_always() -> KeyCode {
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

enum TickOutcome {
    Dead,
    Alive(MoveOutcome),
    Victory,
}

enum GameOutcome {
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

fn recurse(
    g: &mut Graph<Board, i32>,
    board: Board,
    depth: i32,
    winners: &mut Vec<NodeIndex>,
    source_node_index: Option<NodeIndex>,
) {
    if depth >= 1000 {
        println!("Depth too deep, reverting");
        return;
    }

    let current_node_index = g.add_node(board.clone());
    if let Some(source_node_index) = source_node_index {
        g.add_edge(source_node_index, current_node_index, 1);
        println!("Alive(Moved): adding edge {source_node_index:?} -> {current_node_index:?}");
    }

    let offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for offset in offsets {
        println!("At node {current_node_index:?} with depth {depth}");
        println!("{board}");
        get_key();

        let mut board_for_tick = board.clone();
        match tick(&mut board_for_tick, offset) {
            TickOutcome::Dead => {}
            TickOutcome::Alive(MoveOutcome::Moved) => {
                // TODO: Inefficient.
                let mut already_exists = false;
                let h = g.clone();
                let (nodes, _) = h.into_nodes_edges();
                for (index, node) in nodes.iter().enumerate() {
                    let inner_board = node.weight.clone();
                    if inner_board == board_for_tick {
                        g.add_edge(current_node_index, NodeIndex::new(index), 1);
                        already_exists = true;
                    }
                }
                if already_exists {
                    continue;
                }

                println!("Descending into:");
                println!("{board_for_tick}");
                get_key();
                recurse(
                    g,
                    board_for_tick,
                    depth + 1,
                    winners,
                    Some(current_node_index),
                );
            }
            TickOutcome::Victory => {
                println!("Descending into:");
                println!("{board_for_tick}");
                get_key_always();
                let new_node_index = g.add_node(board_for_tick.clone());
                println!("Winner: adding edge {source_node_index:?} -> {current_node_index:?}");
                g.add_edge(current_node_index, new_node_index, 1);
                winners.push(new_node_index);
            }
            TickOutcome::Alive(MoveOutcome::NotMoved) => {}
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.auto {
        let mut board = Board::new_test_01();

        println!("{board}");

        let mut g = Graph::new();
        let mut winners = Vec::new();

        let depth = 0;
        recurse(&mut g, board, depth, &mut winners, None);

        println!("Got {} winners", winners.len());
        if winners.len() == 1 {
            let winner = winners.pop().unwrap();
            let start = g.node_indices().next().unwrap();

            println!("{start:?} -> {winner:?}");

            let path = astar(&g, start, |finish| finish == winner, |_| 1, |_| 0);
            dbg!(&path);

            if let Some((len, nodes)) = path {
                println!("replaying path of len {}:", len);
                nodes.iter().enumerate().for_each(|(index, node)| {
                    let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
                    println!("Step {index}:");
                    println!("{}", g.raw_nodes()[node.index()].weight);
                    println!();
                    get_key_always();
                })
            } else {
                println!("No path?");
            }
        } else {
            println!("No clear winner or no winner at all");
        }
    } else {
        let mut board = Board::new_test_01();
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
}

fn print_board(board: &Board) {
    let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
    println!(
        "Steal the {} from the {}. Use 🡄 🡆 🡅 🡇 to move",
        "$".bright_white(),
        "berserker king".red(),
    );
    println!();
    println!("{board}");
}

fn tick(board: &mut Board, offset: (i32, i32)) -> TickOutcome {
    let outcome = move_player(board, offset);
    if let MoveOutcome::Moved = outcome {
        if is_dead(&*board) {
            return TickOutcome::Dead;
        }
        if is_win(&*board) {
            return TickOutcome::Victory;
        }

        for _ in 0..2 {
            let outcome = move_hunter(board);
            if let MoveOutcome::Moved = outcome {
                if is_dead(&*board) {
                    return TickOutcome::Dead;
                }
            }
        }
        return TickOutcome::Alive(MoveOutcome::Moved);
    }
    TickOutcome::Alive(MoveOutcome::NotMoved)
}
