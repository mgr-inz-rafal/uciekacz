use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    hash::{Hash, Hasher},
    io,
    time::Instant,
};

use crossterm::{
    execute,
    terminal::{self, ClearType},
};
use petgraph::{algo::astar, stable_graph::NodeIndex, Graph};

use crate::{
    board::Board,
    game::{tick, MoveOutcome, TickOutcome},
    utils::get_key,
};

pub(super) fn auto_play(board: Board) {
    print!("Looking for solution... ");

    let start_instant = Instant::now();

    let mut g = Graph::new();
    let mut winners = Vec::new();

    let mut visited = BTreeSet::default();
    let mut hash2index = BTreeMap::default();

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
            println!("Step {}:", index + 1);
            println!("{}", g.raw_nodes()[node.index()].weight);
            println!();
            get_key();
        })
    }
}

fn add_node(
    board: Board,
    already_visited: &mut BTreeSet<u64>,
    hash2index: &mut BTreeMap<u64, NodeIndex>,
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
    hash2index: &mut BTreeMap<u64, NodeIndex>,
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
