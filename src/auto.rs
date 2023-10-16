use std::{
    collections::{hash_map::DefaultHasher, BTreeSet, HashMap},
    hash::{Hash, Hasher},
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};

use crossterm::{
    event::KeyCode,
    execute,
    terminal::{self, ClearType},
};
use multimap::MultiMap;
use petgraph::{algo::astar, stable_graph::NodeIndex, Graph};
use rayon::prelude::{IntoParallelIterator, ParallelBridge, ParallelIterator};

use crate::{
    board::{Board, BoardTensor},
    game::{tick, tick_tensor, MoveOutcome, TickOutcome, TickOutcomeTensor},
    utils::get_key,
};

pub(super) fn auto_play(board: Board) {
    print!("Looking for solution... ");

    let start_instant = Instant::now();

    let mut g = Graph::new();
    let mut winners = Vec::new();

    let mut visited = BTreeSet::default();
    let mut hash2index = HashMap::default();

    let depth = 0;
    recurse(
        &mut g,
        board,
        depth,
        &mut winners,
        None,
        &mut visited,
        &mut hash2index,
    );

    if winners.is_empty() {
        println!("No winner path");
    }

    let paths: BTreeSet<_> = winners
        .iter()
        .map(|winner| {
            let start = g.node_indices().next().unwrap();
            astar(&g, start, |finish| &finish == winner, |_| 1, |_| 0)
                .expect("should have path to winner")
        })
        .collect();

    println!("found in {:?}", start_instant.elapsed());
    if let Some((len, nodes)) = paths.first() {
        println!(
            "Keep pressing any key to reveal the path consisting of {} steps",
            len + 1
        );
        get_key();
        nodes.iter().enumerate().for_each(|(index, node)| {
            let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
            println!("Automatic route: Step {}:", index + 1);
            println!("{}", g.raw_nodes()[node.index()].weight);
            println!();
            get_key();
        })
    }
}

fn add_node(
    board: Board,
    already_visited: &mut BTreeSet<u64>,
    hash2index: &mut HashMap<u64, NodeIndex>,
    g: &mut Graph<Board, i32>,
) -> NodeIndex {
    let mut hasher = DefaultHasher::new();
    board.hash(&mut hasher);
    let hash = hasher.finish();
    already_visited.insert(hash);
    let current_node_index = g.add_node(board.clone());
    hash2index.insert(hash, current_node_index);
    current_node_index
}

fn recurse(
    g: &mut Graph<Board, i32>,
    board: Board,
    depth: i32,
    winners: &mut Vec<NodeIndex>,
    source_node_index: Option<NodeIndex>,
    already_visited: &mut BTreeSet<u64>,
    hash2index: &mut HashMap<u64, NodeIndex>,
) {
    if depth >= 1000 {
        panic!("Recursion too deep, please try with simpler map");
    }

    let current_node_index = add_node(board.clone(), already_visited, hash2index, g);
    if let Some(source_node_index) = source_node_index {
        g.add_edge(source_node_index, current_node_index, 1);
    }

    let offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for offset in offsets {
        let mut next_board = board.clone();
        match tick(&mut next_board, offset) {
            TickOutcome::Dead => {}
            TickOutcome::Alive(MoveOutcome::Moved(_)) => {
                let mut hasher = DefaultHasher::new();
                next_board.hash(&mut hasher);
                let hash = hasher.finish();
                if already_visited.contains(&hash) {
                    g.add_edge(
                        current_node_index,
                        *hash2index.get(&hash).expect("should have index in cache"),
                        1,
                    );
                    continue;
                }

                recurse(
                    g,
                    next_board,
                    depth + 1,
                    winners,
                    Some(current_node_index),
                    already_visited,
                    hash2index,
                );
            }
            TickOutcome::Victory => {
                let new_node_index = add_node(next_board, already_visited, hash2index, g);
                g.add_edge(current_node_index, new_node_index, 1);
                winners.push(new_node_index);
            }
            TickOutcome::Alive(MoveOutcome::NotMoved) => {}
        }
    }
}

#[derive(Clone, Debug)]
struct Route {
    r: Vec<KeyCode>,
}

impl Route {
    fn new(len: usize) -> Self {
        Self {
            r: std::iter::repeat(KeyCode::Left).take(len).collect(),
        }
    }

    pub fn iter(&self) -> RouteIterator {
        RouteIterator::new((*self).clone())
    }
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in &self.r {
            write!(
                f,
                "{}",
                match c {
                    KeyCode::Left => 'L',
                    KeyCode::Right => 'R',
                    KeyCode::Up => 'U',
                    KeyCode::Down => 'D',
                    _ => panic!("unsupported route element: {c:?}"),
                }
            )?
        }
        Ok(())
    }
}

struct RouteIterator {
    current: Route,
    feed_initial: bool,
}

impl RouteIterator {
    fn inc(&mut self, pos: usize) -> bool {
        let c = self.current.r[pos];
        match c {
            KeyCode::Left => self.current.r[pos] = KeyCode::Right,
            KeyCode::Right => self.current.r[pos] = KeyCode::Up,
            KeyCode::Up => self.current.r[pos] = KeyCode::Down,
            KeyCode::Down => {
                self.current.r[pos] = KeyCode::Left;
                if pos < self.current.r.len() - 1 {
                    if !self.inc(pos + 1) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            _ => panic!("unsupported route element: {c:?}"),
        }
        true
    }

    fn advance(&mut self) -> bool {
        self.inc(0)
    }

    fn new(r: Route) -> Self {
        Self {
            current: r,
            feed_initial: true,
        }
    }
}

impl Iterator for RouteIterator {
    type Item = Route;

    fn next(&mut self) -> Option<Self::Item> {
        if self.feed_initial {
            self.feed_initial = false;
            return Some(self.current.clone());
        }
        self.advance().then_some(self.current.clone())
    }
}

impl IntoIterator for Route {
    type Item = Route;
    type IntoIter = RouteIterator;

    fn into_iter(self) -> Self::IntoIter {
        RouteIterator::new(self)
    }
}

#[derive(Debug)]
struct Winner {
    step: usize,
    route: Route,
}

pub(super) fn auto_play_tensor(mut board: BoardTensor) {
    println!("Looking for solution... ");
    const LEN: usize = 2;

    //println!("{board}");

    let start_instant = Instant::now();

    const TOTAL_ROUTES: u64 = 4u64.pow(LEN as u32);
    println!(
        "Total routes to check (millions): {}",
        TOTAL_ROUTES / 1_000_000
    );

    let route = Route::new(LEN);

    for x in route.iter() {
        println!("{x}");
    }
    panic!();

    let current_winner: Arc<Mutex<Option<Winner>>> = Arc::new(Mutex::new(None));
    let counter = AtomicU64::new(0);

    route.into_iter().par_bridge().for_each(|path| {
        let mut next_board = board.clone();
        //        println!("Exercising {path}");
        counter.fetch_add(1, Ordering::Relaxed);
        let current_counter = counter.load(Ordering::Relaxed);
        if current_counter % 1000000 == 0 {
            let cw = current_winner.lock().unwrap();
            match &*cw {
                Some(Winner { step, .. }) => {
                    println!(
                        "At {current_counter} ({:.6}%) the winner is: {}",
                        (100 as f64 * current_counter as f64 / TOTAL_ROUTES as f64),
                        step,
                    )
                }
                None => println!(
                    "At {current_counter} ({:.6}%) there is no winner",
                    (100 as f64 * current_counter as f64 / TOTAL_ROUTES as f64),
                ),
            }
        }
        for (index, step) in path.r.iter().enumerate() {
            let outcome = tick_tensor(&mut next_board, *step);
            match outcome {
                TickOutcomeTensor::Continue => (),
                TickOutcomeTensor::Victory => {
                    let mut cw = current_winner.lock().unwrap();
                    // println!(
                    //     "have a winner using the following path at step {index}:\n{}",
                    //     path.clone(),
                    // );
                    match &mut *cw {
                        None => {
                            // println!(
                            //     "Currently there's no winner, so this is becomes the best one"
                            // );
                            *cw = Some(Winner {
                                step: index,
                                route: path.clone(),
                            });
                        }
                        Some(Winner { step, .. }) => {
                            if &index < step {
                                // println!(
                                //     "And this winner is better than the previous one at {step}"
                                // );
                                *cw = Some(Winner {
                                    step: index,
                                    route: path.clone(),
                                });
                            } else {
                                //println!("But it is worse than the previous one at {step}");
                            }
                        }
                    }
                }
            }
        }
    });

    println!("Solution found in {:?}", start_instant.elapsed());

    let mut cw = current_winner.lock().unwrap();
    match &*cw {
        Some(Winner { step, route }) => {
            println!(
                "Keep pressing any key to reveal the path consisting of {} steps",
                step + 2
            );
            get_key();
            let mut index = 0;
            println!("Step {}/{}...", index + 1, step + 2);
            println!("{board}");
            get_key();
            for action in route.r.iter().take(*step + 1) {
                index += 1;
                let _ = tick_tensor(&mut board, *action);
                println!("Step {}/{}...", index + 1, step + 2);
                println!("{board}");
                get_key();
            }
        }
        None => println!("No winner"),
    }

    // if let Some((len, nodes)) = winner {
    //     println!(
    //         "Keep pressing any key to reveal the path consisting of {} steps",
    //         len + 1
    //     );
    //     get_key();
    //     nodes.iter().enumerate().for_each(|(index, node)| {
    //         let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
    //         println!("Automatic route: Step {}:", index + 1);
    //         println!("{}", g.raw_nodes()[node.index()].weight);
    //         println!();
    //         get_key();
    //     })
    // }

    // print!("\tNow running astar... ");
    // let astart_start = Instant::now();
    // let paths: BTreeSet<_> = winners
    //     .iter()
    //     .map(|winner| {
    //         let start = g.node_indices().next().unwrap();
    //         astar(&g, start, |finish| &finish == winner, |_| 1, |_| 0)
    //             .expect("should have path to winner")
    //     })
    //     .collect();
    // println!("({:?})", astart_start.elapsed());

    // println!("Solution found in {:?}", start_instant.elapsed());
    // if let Some((len, nodes)) = paths.first() {
    //     println!(
    //         "Keep pressing any key to reveal the path consisting of {} steps",
    //         len + 1
    //     );
    //     get_key();
    //     nodes.iter().enumerate().for_each(|(index, node)| {
    //         let _ = execute!(io::stdout(), terminal::Clear(ClearType::All));
    //         println!("Automatic route: Step {}:", index + 1);
    //         println!("{}", g.raw_nodes()[node.index()].weight);
    //         println!();
    //         get_key();
    //     })
    // }
}

use lazy_static::lazy_static;
use std::sync::Mutex;
lazy_static! {
    static ref PREV_TOP: Mutex<usize> = Mutex::new(0);
}

fn recurse_tensor(
    g: &mut Graph<BoardTensor, i32>,
    board: BoardTensor,
    depth: i32,
    score: i32,
    weight: i32,
    winner: &mut Option<(i32, Vec<NodeIndex>)>,
    source_node_index: Option<NodeIndex>,
    already_visited: &mut BTreeSet<u64>,
    hash2index: &mut HashMap<u64, NodeIndex>,
) {
    //println!("depth={depth}");
    if depth >= 50 {
        //println!("{board}");
        return;
    }

    if let Some((current_winner_len, _)) = winner {
        if score >= *current_winner_len {
            //println!("current winner {current_winner_len} is already better or equal than this candidate {score}");
            return;
        }
    }

    if score >= 100 {
        //println!("Score shouldn't exceed 999");
        return;
    }

    // if depth >= 1666 {
    //     let mut prev_access = PREV_TOP.lock().unwrap();
    //     let new_winners = winners.len();
    //     let prev_winners = *prev_access;
    //     if new_winners > prev_winners {
    //         println!(
    //             "Recursion too deep, not exploring further (winners so far={})",
    //             winners.len()
    //         );
    //         *prev_access = new_winners;
    //     }
    //     return;
    // }

    let current_node_index = add_node_tensor(board.clone(), already_visited, hash2index, g);
    if let Some(source_node_index) = source_node_index {
        g.add_edge(source_node_index, current_node_index, weight);
    }

    let offsets = [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down];
    for offset in offsets {
        //println!("BEFORE {score} at {depth}:\n{board}");
        //println!("OFFSET:  {offset:?}");
        let mut next_board = board.clone();
        next_board.transitioned_via = Some(offset);
        let outcome = tick_tensor(&mut next_board, offset);
        //println!("AFTER:\n{next_board}\n\n\n");
        if score == 0 && depth == 0 && offset == KeyCode::Right {
            //println!("trap!");
            //get_key();
        }
        match outcome {
            TickOutcomeTensor::Continue => {
                let mut hasher = DefaultHasher::new();
                next_board.hash(&mut hasher);
                let hash = hasher.finish();
                //dbg!(&hash);

                // g.add_edge(
                //     current_node_index,
                //     *hash2index.get(&hash).expect("should have index in cache"),
                //     1,
                // );

                // if already_visited.contains(&hash) {
                //     g.add_edge(
                //         current_node_index,
                //         *hash2index.get(&hash).expect("should have index in cache"),
                //         1,
                //     );
                //     continue;
                // }

                let weight = match offset {
                    KeyCode::Right | KeyCode::Left => 1,
                    KeyCode::Up | KeyCode::Down => 2,
                    _ => panic!("unsupported offset"),
                };
                recurse_tensor(
                    g,
                    next_board,
                    depth + 1,
                    score + weight,
                    weight,
                    winner,
                    Some(current_node_index),
                    already_visited,
                    hash2index,
                );
            }
            TickOutcomeTensor::Victory => {
                let new_node_index = add_node_tensor(next_board, already_visited, hash2index, g);
                g.add_edge(current_node_index, new_node_index, weight);

                let start = g.node_indices().next().unwrap();
                let (len, path) = astar(
                    &*g,
                    start,
                    |finish| finish == new_node_index,
                    |edge| *edge.weight(),
                    |_| 0,
                )
                .expect("should have path to winner");

                //println!("VICTORY! {len}");
                //get_key();

                match winner {
                    Some((old_len, _)) if len < *old_len => {
                        //println!("old winner ({old_len}) -> new winner ({len}))");
                        //get_key();
                        *winner = Some((len, path))
                    }
                    None => {
                        //println!("new winner ({len}))");
                        //get_key();
                        *winner = Some((len, path))
                    }
                    Some(_) => (),
                }
                return;
            }
        }
    }
}

fn add_node_tensor(
    board: BoardTensor,
    already_visited: &mut BTreeSet<u64>,
    hash2index: &mut HashMap<u64, NodeIndex>,
    g: &mut Graph<BoardTensor, i32>,
) -> NodeIndex {
    let mut hasher = DefaultHasher::new();
    board.hash(&mut hasher);
    let hash = hasher.finish();
    already_visited.insert(hash);
    if hash == 14936445478937173846 {
        //println!("trap inserting hash!");
        //get_key();
    }
    let current_node_index = g.add_node(board.clone());
    hash2index.insert(hash, current_node_index);
    current_node_index
}
