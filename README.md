# tempo

A chess engine in early development.

## Versioning

Tempo uses `0.x` versions as internal development milestones. While the
major version is `0`, there is no promise of public API stability, or even a public API at all. The minor version tracks the next planned engine milestones:

- `0.1.0`: board representation, FEN parsing, move encoding, make/unmake
- `0.2.0`: legal move generation, magic bitboards, and perft verification
- `0.3.0`: UCI support
- `0.4.0`: alpha-beta search with iterative deepening
- `0.5.0`: material counting and PSQT evaluation
- `0.6.0`: time management and stable testing harness

## Status

`0.2.0` is complete, and is focused on correct legal move generation:

- Board representation with mailbox, piece bitboards, and color bitboards
- FEN parsing into position state
- Compact move encoding
- Make/unmake support for quiet moves, captures, en passant, castling, and
  promotions
- Zobrist key maintenance across make/unmake
- Bitboard move generation for pawns, knights, bishops, rooks, queens, and kings
- Magic-bitboard sliding attacks with checked-in, tested lookup metadata
- Legal move filtering through make/unmake
- Perft coverage for start position, Kiwipete, promotions, en passant, castling,
  pins, checks, and king adjacency

Run the test suite:

```sh
cargo test
```

Run perft from the command line:

```sh
cargo run --bin perft -- startpos 4
cargo run --bin perft -- "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1" 4
```
