use super::utils;

mod bitboard;
mod fenparser;
mod makemove;
mod movegen;
mod zobrist;

#[derive(Debug, Default)]
struct Board
{
    pieces: [bitboard::BitBoard; utils::NUM_PIECE_TYPES],
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

#[derive(Debug)]
struct History
{
    arr: [Option<State>; utils::MAX_NUM_PLIES],
    len: usize
}

impl Default for History
{
    fn default() -> Self
    {
        History { arr: [None; utils::MAX_NUM_PLIES], len: 0 }
    }
}

impl History
{
    pub fn push(&mut self, state: State)
    {
        self.arr[self.len] = Some(state);
        self.len += 1;
    }

    pub fn pop(&mut self) -> State
    {
        self.len -= 1;
        self.arr[self.len].unwrap()
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
        let pos = Position::default();
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

    pub fn unmake_move(&mut self)
    {
        makemove::unmake(self);
    }
}
