mod auto;
mod board;
mod game;
mod manual;
mod utils;

use clap::Parser;

use auto::auto_play;
use board::Board;
use manual::manual_play;

#[derive(Parser, Debug)]
struct Args {
    /// If specified, the game will find the solution automatically.
    #[arg(short, long)]
    auto: bool,
    /// Path to file with map. If not provided, default map will be loaded.
    #[arg(short, long)]
    map: Option<String>,
}

fn main() {
    let args = Args::parse();

    let board = args.map.map_or(Board::new_test_01(), Board::from_file);

    if args.auto {
        auto_play(board);
    } else {
        manual_play(board);
    }
}
