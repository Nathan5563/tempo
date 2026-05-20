use super::utils;

mod bitboard;
mod fenparser;
mod makemove;
mod movegen;
mod zobrist;

#[derive(Debug, Default)]
struct Board
{
    pieces: [bitboard::BitBoard; utils::NUM_PIECE_KINDS],
    colors: [bitboard::BitBoard; utils::NUM_COLORS],
    mailbox: utils::Mailbox,
}

#[derive(Debug, Clone, Copy, Default)]
struct State
{
    key: zobrist::ZobristType,
    active: utils::Color,
    castling: utils::CastlingRights,
    enpassant: Option<utils::Square>,
    halfmoves: u8,
    fullmoves: u16,
}

#[derive(Debug, Clone, Copy, Default)]
struct Undo
{
    state: State,
    captured: Option<utils::Piece>,
}

#[derive(Debug)]
struct History
{
    arr: [Undo; utils::MAX_NUM_PLIES],
    len: usize
}

impl Default for History
{
    fn default() -> Self
    {
        Self { arr: [Undo::default(); utils::MAX_NUM_PLIES], len: 0 }
    }
}

impl History
{
    pub fn push(&mut self, undo: Undo)
    {
        self.arr[self.len] = undo;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Undo
    {
        self.len -= 1;
        self.arr[self.len]
    }

    pub fn length(&self) -> usize
    {
        self.len
    }
}

#[derive(Debug, Default)]
pub struct Position
{
    board: Board,
    state: State,
    history: History,
    zobrists: zobrist::ZobristRandoms,
}

impl Position
{
    pub fn new(fen: &str) -> Result<Self, fenparser::Error>
    {
        let pos = Self::default();
        fenparser::parse(fen, &pos)?;
        Ok(pos)
    }

    pub fn generate_moves(&self, movelist: &mut movegen::MoveList)
    {
        movegen::generate(self, movelist);
    }

    pub fn make_move(&mut self, mov: makemove::Move)
    {
        makemove::make(self, mov);
    }

    pub fn unmake_move(&mut self, mov: makemove::Move)
    {
        makemove::unmake(self, mov);
    }
}
