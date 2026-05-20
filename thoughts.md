Action items:
    - Implement making and unmaking moves
    - Implement move generation (magic bitboards?)
    - Test, test, test

A good versioning scheme may be

0.1.0   board representation, FEN parsing, move encoding, make/unmake
0.2.0   legal move generation + perft verified
0.3.0   alpha-beta search with iterative deepening
0.4.0   material counting + PSQT evaluation
0.5.0   UCI support
0.6.0   time management + stable testing harness

From here onward, each version gets its own branch and must pass regression testing. Strength-focused branches merge only if they show a statistically meaningful Elo gain.

0.7.0   transposition table
0.8.0   quiescence search
0.9.0   advanced move ordering + search heuristics
0.10.0  NNUE evaluation

1.0.0   first stable, usable engine as baseline
