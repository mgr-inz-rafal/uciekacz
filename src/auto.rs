use std::{collections::BTreeSet, io, time::Instant};

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

    let depth = 0;
    recurse(&mut g, board, depth, &mut winners, None);

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

fn recurse(
    g: &mut Graph<Board, i32>,
    board: Board,
    depth: i32,
    winners: &mut Vec<NodeIndex>,
    source_node_index: Option<NodeIndex>,
) {
    if depth >= 1000 {
        panic!("Recursion too deep, please try with simpler map");
    }

    let current_node_index = g.add_node(board.clone());
    if let Some(source_node_index) = source_node_index {
        g.add_edge(source_node_index, current_node_index, 1);
    }

    let offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for offset in offsets {
        let mut board_for_tick = board.clone();
        match tick(&mut board_for_tick, offset) {
            TickOutcome::Dead => {}
            TickOutcome::Alive(MoveOutcome::Moved(_)) => {
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

                recurse(
                    g,
                    board_for_tick,
                    depth + 1,
                    winners,
                    Some(current_node_index),
                );
            }
            TickOutcome::Victory => {
                let new_node_index = g.add_node(board_for_tick.clone());
                g.add_edge(current_node_index, new_node_index, 1);
                winners.push(new_node_index);
            }
            TickOutcome::Alive(MoveOutcome::NotMoved) => {}
        }
    }
}
