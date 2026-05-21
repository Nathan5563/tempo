# tempo

A chess engine in early development.

## Versioning

Tempo uses `0.x` versions as internal development milestones. While the
major version is `0`, there is no promise of public API stability, or even a public API at all. The minor version tracks the next planned engine milestones:

- `0.1.0`: board representation, FEN parsing, move encoding, make/unmake
- `0.2.0`: legal move generation and perft verification
- `0.3.0`: alpha-beta search with iterative deepening
- `0.4.0`: material counting and PSQT evaluation
- `0.5.0`: UCI support
- `0.6.0`: time management and stable testing harness

## Status

`0.1.0` is complete, and is focused on the core position layer:

- Board representation with mailbox, piece bitboards, and color bitboards
- FEN parsing into position state
- Compact move encoding
- Make/unmake support for quiet moves, captures, en passant, castling, and
  promotions
- Zobrist key maintenance across make/unmake

Run the test suite with:

```sh
cargo test
```
