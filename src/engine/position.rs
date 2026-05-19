mod bitboard;
mod fenparser;
mod makemove;
mod movegen;
mod utils;

// TODO: Fill out Position struct
#[derive(Debug, Default)]
pub struct Position
{
    // side to move
    // piece-centric structures (bitboard per piece, per color)
    // square-centric structure, mailbox
    // castling rights
    // presence of en passant
    // repetition detection
    // halfmove clock
    // fullmove number
}

impl Position
{
    pub fn new(fen: &str) -> Result<Self, fenparser::Error>
    {
        let pos = Position::default();
        fenparser::parse(fen, &pos)?;
        Ok(pos)
    }
}
