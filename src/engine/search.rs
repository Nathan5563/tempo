mod limits;
mod worker;

use std::sync::atomic::{AtomicBool, Ordering};

use crate::engine::position::{Move, MoveList, Position};

pub use self::limits::SearchLimits;
pub(crate) use self::worker::SearchWorker;

fn search(
    position: &mut Position,
    _limits: &SearchLimits,
    stop: &AtomicBool,
) -> Option<Move>
{
    if stop.load(Ordering::Relaxed)
    {
        return None;
    }
    first_legal_move(position)
}

fn first_legal_move(position: &mut Position) -> Option<Move>
{
    let mut movelist = MoveList::default();
    position.generate_moves(&mut movelist);
    movelist.as_slice().first().copied()
}

#[cfg(test)]
mod tests
{
    use crate::engine::STARTPOS_FEN;

    use super::*;

    #[test]
    fn worker_stop_immediately_returns_legal_fallback()
    {
        let position = Position::new(STARTPOS_FEN).unwrap();
        let worker = SearchWorker::start(position, SearchLimits::default());

        assert_eq!(worker.finish().unwrap().to_string(), "a2a3");
    }
}
