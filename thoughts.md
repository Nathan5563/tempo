init:
- Complete `position/` efficiently
- Complete `evaluate/` with material counts and simple psqt
- Complete `search/` with simple alpha-beta pruning
- Implement uci and divide io/engine threads
- Generally make the engine playable and establish a baseline

(meta)feat:
- Develop some kind of testing framework for correct statistical determination of elo gains
- New features (nnue, quiescence search, transposition tables, etc) should have their own branches, get methodically tested, and only get merged in when they provide sufficient elo benefit
- Figure out rust/cargo versioning so that each new merged branch bumps crate minor version, then once the bot plays sufficiently strong (2000-2500 elo), bump major version to 1.0? Figure it out
