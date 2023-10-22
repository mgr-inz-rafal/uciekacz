mod auto;
mod board;
mod game;
mod manual;
mod utils;

use std::thread;

use clap::Parser;

use auto::{auto_play, auto_play_tensor};
use board::{Board, BoardTensor};
use manual::{manual_play, manual_play_tensor};

#[derive(Parser, Debug)]
struct Args {
    /// If specified, the game will find the solution automatically.
    #[arg(short, long)]
    auto: bool,
    /// If set to 1, search will continue from the saved state (you must provide the same map).
    #[arg(short, long)]
    cont: u8,
    /// Path to file with map. If not provided, default map will be loaded.
    #[arg(short, long)]
    map: Option<String>,
}

fn main() {
    let args = Args::parse();

    let board = args
        .map
        .map_or(BoardTensor::new_test_01(), BoardTensor::from_file);
    let cont = args.cont == 1;

    if args.auto {
        let child = thread::Builder::new()
            .stack_size(1024 * 1024 * 1024 * 50)
            .spawn(move || auto_play_tensor(board, cont))
            .unwrap();
        let _ = child.join();
    } else {
        manual_play_tensor(board);
    }
}
