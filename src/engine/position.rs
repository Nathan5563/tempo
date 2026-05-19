mod bitboard;
mod fenparser;
mod makemove;
mod movegen;
mod utils;

#[derive(Debug, Default)]
struct Board
{
    pieces: [bitboard::BitBoard; utils::NUM_PIECE_TYPES],
    colors: [bitboard::BitBoard; utils::NUM_COLOR_TYPES],
    mailbox: utils::Mailbox,
}

#[derive(Debug, Clone, Copy, Default)]
struct State
{
    key: u64,
    active: utils::Color,
    castling: utils::CastlingRights,
    enpassant: Option<utils::Square>,
    halfmoves: u8,
    fullmoves: u16,
}

#[derive(Debug)]
struct History([State; utils::MAX_NUM_PLIES]);

impl Default for History
{
    fn default() -> Self
    {
        History([State::default(); utils::MAX_NUM_PLIES])
    }
}

#[derive(Debug, Default)]
pub struct Position
{
    board: Board,
    state: State,
    history: History,
    // randomly generated zobrist strings
}

impl Position
{
    pub fn new(fen: &str) -> Result<Self, fenparser::Error>
    {
        let pos = Position::default();
        fenparser::parse(fen, &pos)?;
        Ok(pos)
    }

    pub fn movegen(&self)
    {

    }

    pub fn makemove(&mut self, mov: makemove::Move)
    {
        makemove::make(self, mov);
    }

    pub fn unmakemove(&mut self)
    {
        makemove::unmake(self);
    }
}
