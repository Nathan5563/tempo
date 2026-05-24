pub mod position;
pub mod uci;
mod search;
mod utils;
mod prng;

use self::position::{FenError, Move, MoveList, Position};
use self::search::{SearchLimits, SearchWorker};

const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub enum Error
{
    InvalidFen(FenError),
    IllegalMove(String),
}

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Error::InvalidFen(err) => write!(f, "invalid FEN: {}", err),
            Error::IllegalMove(mov) => write!(f, "illegal move: {}", mov),
        }
    }
}

pub struct Engine
{
    position: Position,
    search: Option<SearchWorker>,
}

impl Engine
{
    pub fn new() -> Self
    {
        Self { position: start_position(), search: None }
    }

    pub fn new_game(&mut self)
    {
        self.cancel_search();
        self.position = start_position();
    }

    pub fn set_position(
        &mut self,
        fen: &str,
        moves: &[&str],
    ) -> std::result::Result<(), Error>
    {
        self.cancel_search();
        let mut position = Position::new(fen).map_err(Error::InvalidFen)?;
        apply_uci_moves(&mut position, moves)?;
        self.position = position;
        Ok(())
    }

    pub fn start_search(&mut self, limits: SearchLimits)
    {
        self.cancel_search();
        self.search =
            Some(SearchWorker::start(self.position.clone(), limits));
    }

    pub fn stop_search(&mut self) -> Option<Option<Move>>
    {
        self.search.take().map(SearchWorker::finish)
    }

    pub fn poll_search(&mut self) -> Option<Option<Move>>
    {
        let result = match self.search.as_mut()
        {
            Some(worker) => worker.try_finish(),
            None => None,
        };

        if result.is_some()
        {
            self.search = None;
        }

        result
    }

    fn cancel_search(&mut self)
    {
        if let Some(worker) = self.search.take()
        {
            let _ = worker.finish();
        }
    }
}

impl Drop for Engine
{
    fn drop(&mut self)
    {
        self.cancel_search();
    }
}

fn start_position() -> Position
{
    Position::new(STARTPOS_FEN).expect("start position FEN must be valid")
}

fn apply_uci_moves(
    position: &mut Position,
    moves: &[&str],
) -> std::result::Result<(), Error>
{
    for text in moves
    {
        let mov = find_uci_move(position, text)
            .ok_or_else(|| Error::IllegalMove((*text).to_owned()))?;
        position.make_move(mov);
    }

    Ok(())
}

fn find_uci_move(position: &mut Position, text: &str) -> Option<Move>
{
    let mut movelist = MoveList::default();
    position.generate_moves(&mut movelist);

    movelist
        .as_slice()
        .iter()
        .copied()
        .find(|mov| mov.to_string() == text)
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn search_bestmove(engine: &mut Engine) -> String
    {
        engine.start_search(SearchLimits::default());
        loop
        {
            if let Some(result) = engine.poll_search()
            {
                return result
                    .map(|mov| mov.to_string())
                    .unwrap_or_else(|| "0000".to_owned());
            }

            std::thread::yield_now();
        }
    }

    #[test]
    fn startpos_search_returns_first_legal_move()
    {
        let mut engine = Engine::new();

        assert_eq!(search_bestmove(&mut engine), "a2a3");
    }

    #[test]
    fn fen_position_searches_from_loaded_position()
    {
        let mut engine = Engine::new();

        engine
            .set_position(
                "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1",
                &[],
            )
            .unwrap();

        assert_eq!(search_bestmove(&mut engine), "e2e3");
    }

    #[test]
    fn set_position_with_moves_accepts_only_legal_moves()
    {
        let mut engine = Engine::new();

        engine.set_position(STARTPOS_FEN, &["e2e4"]).unwrap();
        assert_eq!(search_bestmove(&mut engine), "a7a6");

        assert!(matches!(
            engine.set_position(STARTPOS_FEN, &["e2e5"]),
            Err(Error::IllegalMove(mov)) if mov == "e2e5"
        ));
        assert_eq!(search_bestmove(&mut engine), "a7a6");
    }

    #[test]
    fn set_position_with_moves_commits_only_when_everything_is_valid()
    {
        let mut engine = Engine::new();

        engine.set_position(STARTPOS_FEN, &["e2e4"]).unwrap();
        assert!(
            engine
                .set_position(STARTPOS_FEN, &["e2e4", "e2e5"])
                .is_err()
        );

        assert_eq!(search_bestmove(&mut engine), "a7a6");
    }

    #[test]
    fn new_game_resets_to_startpos()
    {
        let mut engine = Engine::new();

        engine.set_position(STARTPOS_FEN, &["e2e4"]).unwrap();
        engine.new_game();

        assert_eq!(search_bestmove(&mut engine), "a2a3");
    }
}
