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
}

fn main() {
    let args = Args::parse();

    let board = Board::new_test_01();

    if args.auto {
        auto_play(board);
    } else {
        manual_play(board);
    }
}
