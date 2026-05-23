use std::{
    env,
    process,
    time::Instant,
};

use tempo::engine::position::{MoveList, Position};

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main()
{
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2
    {
        eprintln!("usage: cargo run --bin perft -- <fen> <depth>");
        process::exit(2);
    }

    let fen = if args[0] == "startpos" { STARTPOS } else { &args[0] };
    let depth = args[1].parse::<u8>().unwrap_or_else(|_|
    {
        eprintln!("invalid depth: {}", args[1]);
        process::exit(2);
    });

    let mut pos = Position::new(fen).unwrap_or_else(|err|
    {
        eprintln!("invalid FEN: {}", err);
        process::exit(1);
    });

    for depth in 1..=depth
    {
        let start = Instant::now();
        let nodes = perft(&mut pos, depth);

        println!(
            "depth {}: {} nodes ({:.3}s)",
            depth,
            nodes,
            start.elapsed().as_secs_f64(),
        );
    }
}

fn perft(pos: &mut Position, depth: u8) -> u64
{
    if depth == 0
    {
        return 1;
    }

    let mut movelist = MoveList::default();
    pos.generate_moves(&mut movelist);

    if depth == 1
    {
        return movelist.len() as u64;
    }

    let mut nodes = 0;

    for index in 0..movelist.len()
    {
        let mov = movelist.as_slice()[index];

        pos.make_move(mov);
        nodes += perft(pos, depth - 1);
        pos.unmake_move();
    }

    nodes
}
