use std::{
    collections::{hash_map::DefaultHasher, BTreeSet, HashMap},
    hash::{Hash, Hasher},
    io,
    time::Instant,
};

use crossterm::{
    event::KeyCode,
    execute,
    terminal::{self, ClearType},
};
use multimap::MultiMap;
use petgraph::{algo::astar, stable_graph::NodeIndex, Graph};

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

pub(super) fn auto_play_tensor(board: BoardTensor) {
    println!("Looking for solution... ");

    let start_instant = Instant::now();

    //let mut g = Graph::new();
    let mut winner: Option<(i32, Vec<NodeIndex>)> = None;
    let mut depth = 0;
    let mut have = BTreeSet::new();
    let mut tr = MultiMap::new();
    tr.insert(depth, board.clone());
    let mut hasher = DefaultHasher::new();
    board.hash(&mut hasher);
    let hash = hasher.finish();
    have.insert(hash);

    loop {
        // println!(
        //     "boards at depth={depth}: {}",
        //     tr.get_vec(&depth).unwrap().len()
        // );
        //        get_key();
        let boards_at_depth = tr.get_vec(&depth).unwrap().clone();
        println!("{} boards at depth {depth}", boards_at_depth.len());
        for x in boards_at_depth {
            for shift in [KeyCode::Left, KeyCode::Right, KeyCode::Up, KeyCode::Down] {
                let mut new_board = x.clone();
                //                println!("{new_board}");
                //                get_key();
                let victory = match tick_tensor(&mut new_board, shift) {
                    TickOutcomeTensor::Continue => false,
                    TickOutcomeTensor::Victory => true,
                };
                let mut hasher = DefaultHasher::new();
                new_board.hash(&mut hasher);
                let hash = hasher.finish();
                if !have.contains(&hash) {
                    if victory {
                        println!("Victory at depth={depth}!");
                        println!("{new_board}");
                        get_key();
                    }
                    tr.insert(depth + 1, new_board.clone());
                } else {
                    println!("already have!");
                }
                //                println!("Inserting at {}:\n{new_board} from {shift:?}", depth + 1);
                //                get_key();
            }
        }
        depth += 1
    }

    /*
    let mut visited = BTreeSet::default();
    let mut hash2index = HashMap::default();

    let score = 0;
    let depth = 0;
    recurse_tensor(
        &mut g,
        board,
        depth,
        score,
        1,
        &mut winner,
        None,
        &mut visited,
        &mut hash2index,
    );
    */

    // if winner.is_none() {
    //     println!("No winner path");
    //     return;
    // }

    // println!("Solution found in {:?}", start_instant.elapsed());
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
